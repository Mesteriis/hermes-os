use super::*;

impl CommunicationCommandService {
    pub async fn create_folder(
        &self,
        request: NewCommunicationFolder,
    ) -> Result<CommunicationFolder, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder create",
                "COMMUNICATION_FOLDER",
                json!({
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_create",
                }),
                "folder://create".to_owned(),
                json!({
                    "captured_by": "mail_service.create_folder",
                    "operation": "folder_create",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .create_with_observation(
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn update_folder(
        &self,
        folder_id: &str,
        request: UpdateCommunicationFolder,
    ) -> Result<Option<CommunicationFolder>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder update",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "account_id": request.account_id.clone(),
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "color": request.color.clone(),
                    "sort_order": request.sort_order,
                    "operation": "folder_update",
                }),
                format!("folder://{folder_id}/update"),
                json!({
                    "captured_by": "mail_service.update_folder",
                    "operation": "folder_update",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .update_with_observation(
                folder_id,
                request,
                Some(&observation.observation_id),
                "folder_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_folder(
        &self,
        folder_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "folder delete",
                "COMMUNICATION_FOLDER",
                json!({
                    "folder_id": folder_id,
                    "operation": "folder_delete",
                }),
                format!("folder://{folder_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_folder",
                    "operation": "folder_delete",
                }),
            )
            .await?;

        Ok(CommunicationFolderStore::new(self.pool.clone())
            .delete_with_observation(
                folder_id,
                Some(&observation.observation_id),
                "folder_delete",
                None,
            )
            .await?)
    }

    pub async fn copy_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        self.apply_folder_message_action_with_provider_command(
            folder_id,
            message_id,
            crate::domains::communications::folders::FolderMessageOperation::Copy,
        )
        .await
    }

    pub async fn move_message_to_folder(
        &self,
        folder_id: &str,
        message_id: &str,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        self.apply_folder_message_action_with_provider_command(
            folder_id,
            message_id,
            crate::domains::communications::folders::FolderMessageOperation::Move,
        )
        .await
    }

    async fn apply_folder_message_action_with_provider_command(
        &self,
        folder_id: &str,
        message_id: &str,
        operation: crate::domains::communications::folders::FolderMessageOperation,
    ) -> Result<Option<FolderMessageActionResponse>, CommunicationCommandServiceError> {
        let message_store = MessageProjectionStore::new(self.pool.clone());
        let Some(current) = message_store.message(message_id).await? else {
            return Ok(None);
        };
        let operation_name = operation.as_str();
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_MESSAGE",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "folder_id": folder_id,
                    "message_id": message_id,
                    "operation": format!("folder_message_{operation_name}"),
                }),
                format!("folder://{folder_id}/messages/{message_id}/{operation_name}"),
            )
            .provenance(json!({
                "captured_by": "mail_service.folder_message_action",
                "operation": format!("folder_message_{operation_name}"),
            })),
        )
        .await
        .map_err(
            |source| CommunicationCommandServiceError::ObservationCapture {
                operation: "folder message evidence capture",
                source,
            },
        )?;
        let response = CommunicationFolderStore::apply_message_action_in_transaction(
            &mut transaction,
            folder_id,
            message_id,
            operation,
            Some(&observation.observation_id),
            "folder_message_transition",
            None,
        )
        .await?;
        let Some(response) = response else {
            transaction.rollback().await?;
            return Ok(None);
        };

        let mapped_provider_resource = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM communication_mail_provider_resources
                WHERE account_id = $1
                  AND local_folder_id = $2
                  AND writable = true
            )
            "#,
        )
        .bind(&current.account_id)
        .bind(&response.folder_id)
        .fetch_one(&mut *transaction)
        .await?;
        if mapped_provider_resource {
            let command_id = format!("mail-{operation_name}-folder:{}", Uuid::new_v4());
            let command = NewCommunicationProviderCommand::new(
                &command_id,
                &current.account_id,
                "mail",
                match operation_name {
                    "copy" => "copy_folder",
                    "move" => "move_folder",
                    _ => unreachable!("folder operation is closed"),
                },
                &command_id,
                LOCAL_USER_ACTOR_ID,
            )
            .provider_message_id(&current.provider_record_id)
            .target_ref(json!({ "message_id": current.message_id }))
            .payload(json!({
                "folder_id": response.folder_id,
                "provider_record_id": current.provider_record_id,
                "message_metadata": current.message_metadata,
            }));
            CommunicationProviderCommandStore::enqueue_in_transaction(&mut transaction, &command)
                .await?;
        }
        transaction.commit().await?;
        Ok(Some(response))
    }
}
