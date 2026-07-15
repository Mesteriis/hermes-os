use super::*;

impl CommunicationCommandService {
    pub async fn create_saved_search(
        &self,
        request: NewCommunicationSavedSearch,
    ) -> Result<CommunicationSavedSearch, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search create",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "account_id": request.account_id.clone(),
                    "query": request.query.clone(),
                    "workflow_state": request.workflow_state.map(|state| state.as_str().to_owned()),
                    "local_state": request.local_state.map(|state| state.as_str().to_owned()),
                    "channel_kind": request.channel_kind.clone(),
                    "is_smart_folder": request.is_smart_folder,
                    "sort_order": request.sort_order,
                    "operation": "saved_search_create",
                }),
                "saved-search://create".to_owned(),
                json!({
                    "captured_by": "mail_service.create_saved_search",
                    "operation": "saved_search_create",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .create_with_observation(
                request,
                Some(&observation.observation_id),
                "saved_search_upsert",
                None,
            )
            .await?)
    }
    pub async fn update_saved_search(
        &self,
        saved_search_id: &str,
        request: UpdateCommunicationSavedSearch,
    ) -> Result<Option<CommunicationSavedSearch>, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search update",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "saved_search_id": saved_search_id,
                    "name": request.name.clone(),
                    "description": request.description.clone(),
                    "account_id": request.account_id.clone(),
                    "query": request.query.clone(),
                    "workflow_state": request.workflow_state.map(|state| state.as_str().to_owned()),
                    "local_state": request.local_state.map(|state| state.as_str().to_owned()),
                    "channel_kind": request.channel_kind.clone(),
                    "is_smart_folder": request.is_smart_folder,
                    "sort_order": request.sort_order,
                    "operation": "saved_search_update",
                }),
                format!("saved-search://{saved_search_id}/update"),
                json!({
                    "captured_by": "mail_service.update_saved_search",
                    "operation": "saved_search_update",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .update_with_observation(
                saved_search_id,
                request,
                Some(&observation.observation_id),
                "saved_search_upsert",
                None,
            )
            .await?)
    }

    pub async fn delete_saved_search(
        &self,
        saved_search_id: &str,
    ) -> Result<bool, CommunicationCommandServiceError> {
        let observation = self
            .capture_observation(
                "saved search delete",
                "COMMUNICATION_SAVED_SEARCH",
                json!({
                    "saved_search_id": saved_search_id,
                    "operation": "saved_search_delete",
                }),
                format!("saved-search://{saved_search_id}/delete"),
                json!({
                    "captured_by": "mail_service.delete_saved_search",
                    "operation": "saved_search_delete",
                }),
            )
            .await?;

        Ok(CommunicationSavedSearchStore::new(self.pool.clone())
            .delete_with_observation(
                saved_search_id,
                Some(&observation.observation_id),
                "saved_search_delete",
                None,
            )
            .await?)
    }
}
