// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{BanAvatarResult, BanCheckResult, BulkBanResult};

impl AdminApiClient {
    pub async fn ban_email(&self, email: &str) -> ApiResult<()> {
        let body = generated_types::BanEmailRequest {
            email: generated_types::EmailType::from(email.to_owned()),
        };
        self.generated()
            .add_email_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_email(&self, email: &str) -> ApiResult<()> {
        let body = generated_types::BanEmailRequest {
            email: generated_types::EmailType::from(email.to_owned()),
        };
        self.generated()
            .remove_email_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_email_ban(&self, email: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::BanEmailRequest {
            email: generated_types::EmailType::from(email.to_owned()),
        };
        let response = self
            .generated()
            .check_email_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_ip(&self, ip: &str) -> ApiResult<()> {
        let body = generated_types::BanIpRequest { ip: ip.to_owned() };
        self.generated()
            .add_ip_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_ip(&self, ip: &str) -> ApiResult<()> {
        let body = generated_types::BanIpRequest { ip: ip.to_owned() };
        self.generated()
            .remove_ip_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_ip_ban(&self, ip: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::BanIpRequest { ip: ip.to_owned() };
        let response = self
            .generated()
            .check_ip_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn add_suspicious_email_domain(&self, domain: &str) -> ApiResult<()> {
        let body = suspicious_email_domain_request(domain)?;
        self.generated()
            .add_suspicious_email_domain(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn remove_suspicious_email_domain(&self, domain: &str) -> ApiResult<()> {
        let body = suspicious_email_domain_request(domain)?;
        self.generated()
            .remove_suspicious_email_domain(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_suspicious_email_domain(&self, domain: &str) -> ApiResult<BanCheckResult> {
        let body = suspicious_email_domain_request(domain)?;
        let response = self
            .generated()
            .check_suspicious_email_domain(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_phrase(&self, phrase: &str) -> ApiResult<()> {
        let body = generated_types::BanPhraseRequest {
            phrase: phrase.to_owned(),
        };
        self.generated()
            .add_phrase_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_phrase(&self, phrase: &str) -> ApiResult<()> {
        let body = generated_types::BanPhraseRequest {
            phrase: phrase.to_owned(),
        };
        self.generated()
            .remove_phrase_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_phrase_ban(&self, phrase: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::BanPhraseRequest {
            phrase: phrase.to_owned(),
        };
        let response = self
            .generated()
            .check_phrase_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_url(&self, url: &str) -> ApiResult<()> {
        let body = generated_types::BanUrlRequest {
            category: None,
            notes: None,
            severity: None,
            source_url: None,
            url: url.to_owned(),
        };
        self.generated()
            .add_url_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_url(&self, url: &str) -> ApiResult<()> {
        let body = generated_types::UnbanUrlRequest {
            url: url.to_owned(),
        };
        self.generated()
            .remove_url_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_url_ban(&self, url: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::CheckUrlBlocklistRequest {
            url: url.to_owned(),
        };
        let response = self
            .generated()
            .check_url_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_url_domain(&self, domain: &str, match_subdomains: bool) -> ApiResult<()> {
        let body = generated_types::BanUrlDomainRequest {
            category: None,
            domain: domain.to_owned(),
            match_subdomains: Some(match_subdomains),
            notes: None,
            severity: None,
            source_url: None,
        };
        self.generated()
            .add_url_domain_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_url_domain(&self, domain: &str) -> ApiResult<()> {
        let body = generated_types::UnbanUrlDomainRequest {
            domain: domain.to_owned(),
        };
        self.generated()
            .remove_url_domain_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_url_domain_ban(&self, domain: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::BanUrlDomainRequest {
            category: None,
            domain: domain.to_owned(),
            match_subdomains: None,
            notes: None,
            severity: None,
            source_url: None,
        };
        let response = self
            .generated()
            .check_url_domain_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_file_sha(
        &self,
        sha256_hex: &str,
        audit_log_reason: Option<&str>,
    ) -> ApiResult<()> {
        let body = generated_types::BanFileShaRequest {
            category: None,
            content_type: None,
            notes: None,
            severity: None,
            sha256_hex: sha256_hex.to_owned(),
            source_url: None,
        };
        self.post_typed_with_reason::<(), _>("/admin/bans/file-sha/add", &body, audit_log_reason)
            .await
    }

    pub async fn unban_file_sha(
        &self,
        sha256_hex: &str,
        audit_log_reason: Option<&str>,
    ) -> ApiResult<()> {
        let body = generated_types::UnbanFileShaRequest {
            sha256_hex: sha256_hex.to_owned(),
        };
        self.post_typed_with_reason::<(), _>("/admin/bans/file-sha/remove", &body, audit_log_reason)
            .await
    }

    pub async fn check_file_sha_ban(&self, sha256_hex: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::CheckFileShaRequest {
            sha256_hex: sha256_hex.to_owned(),
        };
        let response = self
            .generated()
            .check_file_sha_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn bulk_ban_file_shas(
        &self,
        sha256_list: &[String],
        audit_log_reason: Option<&str>,
    ) -> ApiResult<BulkBanResult> {
        let body = generated_types::BulkBanFileShasRequest {
            sha256_list: sha256_list.to_vec(),
        };
        self.post_typed_with_reason("/admin/bans/file-sha/bulk-add", &body, audit_log_reason)
            .await
    }

    pub async fn ban_avatar_hash(&self, hash_short: &str) -> ApiResult<()> {
        let body = generated_types::BanAvatarHashRequest {
            category: None,
            hashes: vec![hash_short.to_owned()],
            notes: None,
            reason: None,
            severity: None,
            source_url: None,
        };
        self.generated()
            .add_avatar_hash_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_avatar_hash(&self, hash_short: &str) -> ApiResult<()> {
        let body = generated_types::CheckAvatarHashRequest {
            hashes: vec![hash_short.to_owned()],
        };
        self.generated()
            .remove_avatar_hash_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_avatar_hash_ban(&self, hash_short: &str) -> ApiResult<BanCheckResult> {
        let body = generated_types::CheckAvatarHashRequest {
            hashes: vec![hash_short.to_owned()],
        };
        let response = self
            .generated()
            .check_avatar_hash_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_user_avatar(&self, user_id: &str) -> ApiResult<BanAvatarResult> {
        let body = generated_types::BanUserAvatarRequest::default();
        let response = self
            .generated()
            .ban_user_avatar(
                &generated_types::SnowflakeType::from(user_id.to_owned()),
                &body,
            )
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn ban_profile_substring(&self, scope: &str, substring: &str) -> ApiResult<()> {
        let body = profile_substring_request(scope, substring)?;
        self.generated()
            .add_profile_substring_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn unban_profile_substring(&self, scope: &str, substring: &str) -> ApiResult<()> {
        let body = profile_substring_request(scope, substring)?;
        self.generated()
            .remove_profile_substring_ban(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        Ok(())
    }

    pub async fn check_profile_substring_ban(
        &self,
        scope: &str,
        substring: &str,
    ) -> ApiResult<BanCheckResult> {
        let body = profile_substring_request(scope, substring)?;
        let response = self
            .generated()
            .check_profile_substring_ban_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }
}

fn suspicious_email_domain_request(
    domain: &str,
) -> ApiResult<generated_types::SuspiciousEmailDomainRequest> {
    Ok(generated_types::SuspiciousEmailDomainRequest {
        domain: generated_types::SuspiciousEmailDomainRequestDomain::try_from(domain)
            .map_err(|e| ApiError::Parse(e.to_string()))?,
    })
}

fn profile_substring_request(
    scope: &str,
    substring: &str,
) -> ApiResult<generated_types::BanProfileSubstringRequest> {
    Ok(generated_types::BanProfileSubstringRequest {
        notes: None,
        reason: None,
        scope: generated_types::BanProfileSubstringRequestScope::try_from(scope)
            .map_err(|e| ApiError::Parse(e.to_string()))?,
        substrings: vec![substring.to_owned()],
    })
}
