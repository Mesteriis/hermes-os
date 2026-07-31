use hermes_ollama_ai_core::{OllamaGenerationPlanV1, OllamaHttpGenerationV1};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OllamaAiHttpErrorV1 {
    InvalidConfiguration,
    InvalidRequest,
    Unavailable,
    Rejected,
    Protocol,
    ModelUnavailable,
    ModelMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaModelRevisionV1 {
    pub model: String,
    pub digest: [u8; 32],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TagsResponseV1 {
    models: Vec<TagModelV1>,
}

#[derive(Deserialize)]
struct TagModelV1 {
    name: String,
    model: String,
    digest: String,
}

#[derive(Serialize)]
struct ChatRequestV1<'a> {
    model: &'a str,
    messages: [ChatMessageV1<'a>; 1],
    stream: bool,
    think: bool,
    format: ReplyJsonSchemaV1,
    options: ChatOptionsV1,
}

#[derive(Serialize)]
struct ReplyJsonSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: ReplyJsonPropertiesV1,
    required: [&'static str; 3],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

#[derive(Serialize)]
struct ReplyJsonPropertiesV1 {
    subject: JsonStringSchemaV1,
    body: JsonStringSchemaV1,
    language: JsonLanguageSchemaV1,
}

#[derive(Serialize)]
struct JsonStringSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct JsonLanguageSchemaV1 {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "enum")]
    allowed: [&'static str; 3],
}

#[derive(Serialize)]
struct ChatMessageV1<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatOptionsV1 {
    temperature: u8,
    num_predict: u32,
}

#[derive(Deserialize)]
struct ChatResponseV1 {
    model: String,
    message: ChatResponseMessageV1,
    done: bool,
    prompt_eval_count: u32,
    eval_count: u32,
}

#[derive(Deserialize)]
struct ChatResponseMessageV1 {
    role: String,
    content: String,
    #[serde(default)]
    thinking: String,
}

pub(crate) fn decode_model_revision_v1(
    body: &[u8],
    selected_model: &str,
) -> Result<OllamaModelRevisionV1, OllamaAiHttpErrorV1> {
    let response: TagsResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    let mut matches = response
        .models
        .into_iter()
        .filter(|candidate| candidate.name == selected_model || candidate.model == selected_model);
    let model = matches
        .next()
        .ok_or(OllamaAiHttpErrorV1::ModelUnavailable)?;
    if matches.next().is_some() {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaModelRevisionV1 {
        model: selected_model.to_owned(),
        digest: decode_sha256_hex_v1(&model.digest)?,
    })
}

pub(crate) fn encode_chat_request_v1(
    plan: &OllamaGenerationPlanV1,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let prompt =
        std::str::from_utf8(&plan.prompt_utf8).map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)?;
    serde_json::to_vec(&ChatRequestV1 {
        model: &plan.model,
        messages: [ChatMessageV1 {
            role: "user",
            content: prompt,
        }],
        stream: false,
        think: false,
        format: reply_json_schema_v1(),
        options: ChatOptionsV1 {
            temperature: 0,
            num_predict: plan.maximum_output_tokens,
        },
    })
    .map(Zeroizing::new)
    .map_err(|_| OllamaAiHttpErrorV1::InvalidRequest)
}

fn reply_json_schema_v1() -> ReplyJsonSchemaV1 {
    ReplyJsonSchemaV1 {
        kind: "object",
        properties: ReplyJsonPropertiesV1 {
            subject: JsonStringSchemaV1 { kind: "string" },
            body: JsonStringSchemaV1 { kind: "string" },
            language: JsonLanguageSchemaV1 {
                kind: "string",
                allowed: ["english", "spanish", "russian"],
            },
        },
        required: ["subject", "body", "language"],
        additional_properties: false,
    }
}

pub(crate) fn decode_chat_response_v1(
    body: &[u8],
    plan: &OllamaGenerationPlanV1,
) -> Result<OllamaHttpGenerationV1, OllamaAiHttpErrorV1> {
    let mut response: ChatResponseV1 =
        serde_json::from_slice(body).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    if response.model != plan.model
        || !response.done
        || response.message.role != "assistant"
        || !response.message.thinking.is_empty()
        || response.message.content.is_empty()
        || response.message.content.len() > plan.maximum_output_bytes as usize
    {
        return Err(OllamaAiHttpErrorV1::ModelMismatch);
    }
    Ok(OllamaHttpGenerationV1 {
        content_json_utf8: Zeroizing::new(
            std::mem::take(&mut response.message.content).into_bytes(),
        ),
        model_digest: plan.model_digest,
        input_tokens: response.prompt_eval_count,
        output_tokens: response.eval_count,
    })
}

fn decode_sha256_hex_v1(value: &str) -> Result<[u8; 32], OllamaAiHttpErrorV1> {
    if value.len() != 64 {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    let mut digest = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        digest[index] = (hex_nibble_v1(chunk[0])? << 4) | hex_nibble_v1(chunk[1])?;
    }
    (digest != [0; 32])
        .then_some(digest)
        .ok_or(OllamaAiHttpErrorV1::Protocol)
}

fn hex_nibble_v1(value: u8) -> Result<u8, OllamaAiHttpErrorV1> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(OllamaAiHttpErrorV1::Protocol),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_model_discovery_decodes_revision() {
        let body = br#"{"models":[{"name":"gemma3:latest","model":"gemma3:latest","digest":"0909090909090909090909090909090909090909090909090909090909090909"}]}"#;
        assert_eq!(
            decode_model_revision_v1(body, "gemma3:latest"),
            Ok(OllamaModelRevisionV1 {
                model: "gemma3:latest".to_owned(),
                digest: [9; 32],
            })
        );
    }

    #[test]
    fn reply_schema_is_closed_and_language_bounded() {
        assert_eq!(
            serde_json::to_string(&reply_json_schema_v1()).expect("reply JSON schema"),
            r#"{"type":"object","properties":{"subject":{"type":"string"},"body":{"type":"string"},"language":{"type":"string","enum":["english","spanish","russian"]}},"required":["subject","body","language"],"additionalProperties":false}"#
        );
    }
}
