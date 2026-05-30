use async_trait::async_trait;
use serde_json::Value;

/// The request structure passed to the HttpClient.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub form: Vec<(String, String)>,
    pub json: Option<Value>,
    pub basic_auth: Option<(String, Option<String>)>,
    pub bearer_auth: Option<String>,
}

/// The response structure returned by the HttpClient.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Value,
}

/// The trait that custom HTTP clients must implement.
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, req: HttpRequest) -> Result<HttpResponse, crate::error::ConnectError>;
}

/// Extension trait to provide the fluent builder API (like reqwest).
pub trait HttpClientExt {
    fn get(&self, url: impl Into<String>) -> RequestBuilder<'_>;
    fn post(&self, url: impl Into<String>) -> RequestBuilder<'_>;
}

impl HttpClientExt for dyn HttpClient + '_ {
    fn get(&self, url: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder::new(self, "GET".to_string(), url.into())
    }
    fn post(&self, url: impl Into<String>) -> RequestBuilder<'_> {
        RequestBuilder::new(self, "POST".to_string(), url.into())
    }
}

/// A fluent builder for HTTP requests, matching the subset of reqwest used by providers.
pub struct RequestBuilder<'a> {
    client: &'a dyn HttpClient,
    req: HttpRequest,
}

impl<'a> RequestBuilder<'a> {
    pub fn new(client: &'a dyn HttpClient, method: String, url: String) -> Self {
        Self {
            client,
            req: HttpRequest {
                method,
                url,
                headers: vec![],
                form: vec![],
                json: None,
                basic_auth: None,
                bearer_auth: None,
            },
        }
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.req.headers.push((key.into(), value.into()));
        self
    }

    pub fn bearer_auth(mut self, token: &str) -> Self {
        self.req.bearer_auth = Some(token.to_string());
        self
    }

    pub fn basic_auth(mut self, username: &str, password: Option<&str>) -> Self {
        self.req.basic_auth = Some((username.to_string(), password.map(|s| s.to_string())));
        self
    }

    pub fn json(mut self, value: &Value) -> Self {
        self.req.json = Some(value.clone());
        self
    }

    pub fn form<K, V>(mut self, form: &[(K, V)]) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (k, v) in form {
            self.req
                .form
                .push((k.as_ref().to_string(), v.as_ref().to_string()));
        }
        self
    }

    pub async fn send(self) -> Result<ResponseWrapper, crate::error::ConnectError> {
        let res = self.client.execute(self.req).await?;
        Ok(ResponseWrapper { res })
    }
}

pub struct ResponseWrapper {
    res: HttpResponse,
}

impl ResponseWrapper {
    pub fn error_for_status(self) -> Result<Self, crate::error::ConnectError> {
        if self.res.status >= 400 {
            Err(crate::error::ConnectError::Provider(format!(
                "HTTP Error: {}",
                self.res.status
            )))
        } else {
            Ok(self)
        }
    }

    pub async fn json<T>(self) -> Result<T, crate::error::ConnectError>
    where
        T: serde::de::DeserializeOwned,
    {
        let t = serde_json::from_value(self.res.body)?;
        Ok(t)
    }
}

/// The default reqwest-based implementation of `HttpClient`.
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn execute(&self, req: HttpRequest) -> Result<HttpResponse, crate::error::ConnectError> {
        let method = match req.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            _ => reqwest::Method::GET,
        };

        let mut builder = self.client.request(method, &req.url);

        for (k, v) in &req.headers {
            builder = builder.header(k, v);
        }

        if let Some(token) = &req.bearer_auth {
            builder = builder.bearer_auth(token);
        }

        if let Some((user, pass)) = &req.basic_auth {
            builder = builder.basic_auth(user, pass.as_deref());
        }

        if !req.form.is_empty() {
            builder = builder.form(&req.form);
        } else if let Some(j) = &req.json {
            builder = builder.json(j);
        }

        let res = builder.send().await?;
        let status = res.status().as_u16();
        // Read body as text first in case it's not JSON
        let text = res.text().await?;
        let body = serde_json::from_str(&text).unwrap_or(Value::String(text));

        Ok(HttpResponse { status, body })
    }
}
