// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{
    BillingOverview, InvoiceListResponse, PaymentListResponse, PaymentMethodListResponse,
    RefundCancelResponse, SubscriptionResponse,
};

impl AdminApiClient {
    pub async fn get_billing_overview(&self, user_id: &str) -> ApiResult<BillingOverview> {
        let response = self
            .generated()
            .admin_billing_overview(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_user_payments(&self, user_id: &str) -> ApiResult<PaymentListResponse> {
        let response = self
            .generated()
            .admin_billing_list_payments(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_user_subscription(&self, user_id: &str) -> ApiResult<SubscriptionResponse> {
        let response = self
            .generated()
            .admin_billing_get_subscription(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_user_payment_methods(
        &self,
        user_id: &str,
    ) -> ApiResult<PaymentMethodListResponse> {
        let response = self
            .generated()
            .admin_billing_list_payment_methods(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_user_invoices(
        &self,
        user_id: &str,
        limit: u32,
        starting_after: Option<&str>,
    ) -> ApiResult<InvoiceListResponse> {
        let limit_str = limit.to_string();
        let mut params: Vec<(&str, &str)> = vec![("limit", &limit_str)];
        if let Some(sa) = starting_after {
            params.push(("starting_after", sa));
        }
        self.get(
            &format!("/admin/billing/users/{user_id}/invoices"),
            Some(&params),
        )
        .await
    }

    pub async fn issue_refund(
        &self,
        user_id: &str,
        payment_intent_id: &str,
        amount_cents: Option<u64>,
        reason: Option<&str>,
    ) -> ApiResult<()> {
        let body = generated_types::AdminBillingRefundRequest {
            amount_cents: amount_cents
                .map(|value| crate::api::generated::nonzero_u64(value, "amount_cents"))
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
            payment_intent_id: payment_intent_id.to_owned(),
            reason: reason
                .map(generated_types::AdminBillingRefundRequestReason::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
        };
        self.generated()
            .admin_billing_refund(user_id, &body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn refund_policy_cancel_now(
        &self,
        user_id: &str,
        reason: Option<&str>,
    ) -> ApiResult<RefundCancelResponse> {
        let body = generated_types::AdminBillingRefundLatestInvoiceCancelRequest {
            reason: reason
                .map(generated_types::AdminBillingRefundLatestInvoiceCancelRequestReason::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
        };
        let response = self
            .generated()
            .admin_billing_refund_policy_cancel_now(user_id, &body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn cancel_subscription(&self, user_id: &str) -> ApiResult<()> {
        self.generated()
            .admin_billing_cancel_subscription(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn cancel_subscription_immediately(
        &self,
        user_id: &str,
        reason: Option<&str>,
    ) -> ApiResult<()> {
        let body = generated_types::AdminBillingCancelImmediatelyRequest {
            reason: reason
                .map(generated_types::AdminBillingCancelImmediatelyRequestReason::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
        };
        self.generated()
            .admin_billing_cancel_subscription_now(user_id, &body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn reactivate_subscription(&self, user_id: &str) -> ApiResult<()> {
        self.generated()
            .admin_billing_reactivate_subscription(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn end_premium_grace_period(&self, user_id: &str) -> ApiResult<()> {
        self.generated()
            .admin_billing_end_premium_grace_period(user_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }
}
