// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use maud::{DOCTYPE, html};

pub struct AppError {
    pub status: StatusCode,
    pub message: String,
    pub detail: Option<String>,
    pub home_url: String,
}

impl AppError {
    pub fn not_found(message: &str, base_path: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_owned(),
            detail: None,
            home_url: if base_path.is_empty() {
                "/".to_owned()
            } else {
                base_path.to_owned()
            },
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_owned(),
            detail: None,
            home_url: "/login".to_owned(),
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.to_owned(),
            detail: None,
            home_url: "/login".to_owned(),
        }
    }

    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn with_base_path(mut self, base_path: &str) -> Self {
        self.home_url = if base_path.is_empty() {
            "/".to_owned()
        } else {
            base_path.to_owned()
        };
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status.as_u16();
        let status_text = self.status.canonical_reason().unwrap_or("Error");
        let home_url = &self.home_url;
        let (border, bg, text_color) = if status_code >= 500 {
            ("border-red-300", "bg-red-50", "text-red-800")
        } else if status_code == 403 {
            ("border-yellow-300", "bg-yellow-50", "text-yellow-800")
        } else {
            ("border-neutral-300", "bg-neutral-50", "text-neutral-800")
        };
        let markup = html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { (status_code) " " (status_text) " ~ Fluxer Admin" }
                }
                body class="min-h-screen bg-neutral-50 flex items-center justify-center" {
                    div class="mx-auto max-w-lg px-4 py-16 text-center" {
                        h1 class="text-6xl font-bold text-neutral-300 mb-4" {
                            (status_code)
                        }
                        h2 class="text-xl font-semibold text-neutral-700 mb-2" {
                            (status_text)
                        }
                        div class={"rounded-lg border px-4 py-3 text-sm mb-6 " (border) " " (bg) " " (text_color)} {
                            div { (self.message) }
                            @if let Some(ref detail) = self.detail {
                                div class="mt-2 break-all rounded border border-current/20 \
                                           bg-white/60 px-3 py-2 text-xs" {
                                    (detail)
                                }
                            }
                        }
                        a href=(home_url)
                            class="text-blue-600 hover:text-blue-800 hover:underline text-sm" {
                            "Go to admin"
                        }
                    }
                }
            }
        };
        (self.status, Html(markup.into_string())).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(?err, "internal server error");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "An internal error occurred.".to_owned(),
            detail: None,
            home_url: "/login".to_owned(),
        }
    }
}
