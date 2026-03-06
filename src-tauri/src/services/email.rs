use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::emailpb;
use crate::state::AppState;

pub struct EmailService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
}

impl EmailService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self { network, state }
    }

    /// Get email list
    pub async fn get_email_list(&self, box_type: i32) -> AppResult<emailpb::GetEmailListReply> {
        let req = emailpb::GetEmailListRequest { box_type };
        let reply_bytes = self
            .network
            .send_request(&codec::GET_EMAIL_LIST, req.encode_to_vec())
            .await?;
        Ok(emailpb::GetEmailListReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim a single email
    pub async fn claim_email(
        &self,
        box_type: i32,
        email_id: &str,
    ) -> AppResult<emailpb::ClaimEmailReply> {
        let req = emailpb::ClaimEmailRequest {
            box_type,
            email_id: email_id.to_string(),
        };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_EMAIL, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.emails_claimed += 1);
        Ok(emailpb::ClaimEmailReply::decode(reply_bytes.as_slice())?)
    }

    /// Batch claim emails
    pub async fn batch_claim_email(
        &self,
        box_type: i32,
        email_id: &str,
    ) -> AppResult<emailpb::BatchClaimEmailReply> {
        let req = emailpb::BatchClaimEmailRequest {
            box_type,
            email_id: email_id.to_string(),
        };
        let reply_bytes = self
            .network
            .send_request(&codec::BATCH_CLAIM_EMAIL, req.encode_to_vec())
            .await?;
        Ok(emailpb::BatchClaimEmailReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Auto-claim all emails with rewards
    pub async fn auto_claim_all(&self) -> AppResult<()> {
        for box_type in [1, 2] {
            let reply = self.get_email_list(box_type).await?;

            for email in &reply.emails {
                if email.has_reward && !email.claimed {
                    log::info!("Claiming email: {}", email.title);
                    let _ = self.claim_email(box_type, &email.id).await;
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }
        }

        Ok(())
    }
}
