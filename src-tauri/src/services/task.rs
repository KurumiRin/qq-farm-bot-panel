use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::taskpb;
use crate::state::AppState;

pub struct TaskService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
}

impl TaskService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self { network, state }
    }

    /// Get all task info
    pub async fn get_task_info(&self) -> AppResult<taskpb::TaskInfoReply> {
        let req = taskpb::TaskInfoRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::TASK_INFO, req.encode_to_vec())
            .await?;
        Ok(taskpb::TaskInfoReply::decode(reply_bytes.as_slice())?)
    }

    /// Claim a single task reward
    pub async fn claim_task_reward(
        &self,
        task_id: i64,
        do_shared: bool,
    ) -> AppResult<taskpb::ClaimTaskRewardReply> {
        let req = taskpb::ClaimTaskRewardRequest {
            id: task_id,
            do_shared,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_TASK_REWARD, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.tasks_claimed += 1);
        Ok(taskpb::ClaimTaskRewardReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Batch claim task rewards
    pub async fn batch_claim_task_reward(
        &self,
        ids: Vec<i64>,
        do_shared: bool,
    ) -> AppResult<taskpb::BatchClaimTaskRewardReply> {
        let req = taskpb::BatchClaimTaskRewardRequest { ids, do_shared };
        let reply_bytes = self
            .network
            .send_request(&codec::BATCH_CLAIM_TASK_REWARD, req.encode_to_vec())
            .await?;
        Ok(taskpb::BatchClaimTaskRewardReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim daily activity reward
    pub async fn claim_daily_reward(
        &self,
        active_type: i32,
        point_ids: Vec<i64>,
    ) -> AppResult<taskpb::ClaimDailyRewardReply> {
        let req = taskpb::ClaimDailyRewardRequest {
            r#type: active_type,
            point_ids,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_DAILY_REWARD, req.encode_to_vec())
            .await?;
        Ok(taskpb::ClaimDailyRewardReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Auto-claim all claimable tasks and activity rewards
    pub async fn auto_claim_all(&self) -> AppResult<()> {
        let reply = self.get_task_info().await?;
        let task_info = match reply.task_info {
            Some(info) => info,
            None => return Ok(()),
        };

        // Collect claimable task IDs
        let mut claimable_ids = Vec::new();
        let all_tasks = task_info
            .growth_tasks
            .iter()
            .chain(task_info.daily_tasks.iter())
            .chain(task_info.tasks.iter());

        for task in all_tasks {
            if !task.is_claimed && task.is_unlocked && task.progress >= task.total_progress {
                claimable_ids.push(task.id);
            }
        }

        // Batch claim tasks
        if !claimable_ids.is_empty() {
            log::info!("Claiming {} tasks", claimable_ids.len());
            let _ = self
                .batch_claim_task_reward(claimable_ids, false)
                .await;
        }

        // Claim activity rewards
        for active in &task_info.actives {
            let claimable_points: Vec<i64> = active
                .rewards
                .iter()
                .filter(|r| r.status == 2 && r.need_progress <= active.progress)
                .map(|r| r.point_id)
                .collect();

            if !claimable_points.is_empty() {
                log::info!(
                    "Claiming {} activity rewards (type={})",
                    claimable_points.len(),
                    active.r#type
                );
                let _ = self
                    .claim_daily_reward(active.r#type, claimable_points)
                    .await;
            }
        }

        Ok(())
    }
}
