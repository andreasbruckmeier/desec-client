use reqwest::{ header, Response, Error };
use serde::{Deserialize, Serialize};
use thiserror::Error;

static API_URL: &str = "https://desec.io/api/v1";

#[derive(Error, Debug)]
pub enum DeSecError {
    #[error("An http error occured: {0}")]
    Http(String, reqwest::Error),
    #[error("{0}")]
    NotFound(String),
    #[error("The requet resource does not exist: {0}")]
    Parser(String),
    #[error("Failed to create HTTP client: {0}")]
    ClientBuilder(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInformation {
    pub created: String,
    pub email: String,
    pub id: String,
    pub limit_domains: u64,
    pub outreach_preference: bool
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Domain {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<DNSSECKeyInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_ttl: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zonefile: Option<String>
}

type DomainList = Vec<Domain>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DNSSECKeyInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dnskey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ds: Option<Vec<String>>,
    #[serde(rename = "flags")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyflags: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keytype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResourceRecordSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rrset_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touched: Option<String>
}

type ResourceRecordSetList = Vec<ResourceRecordSet>;

#[derive(Debug, Clone)]
pub struct DeSecClient {
    client: reqwest::Client,
    pub api_url: String,
    pub token: String
}

impl DeSecClient {

    pub fn new(token: String) -> Result<Self, DeSecError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(
                format!("Token {}", token.as_str()).as_str()
            ).unwrap()
        );
        let client = reqwest::ClientBuilder::new()
            .user_agent("rust-desec-client")
            .default_headers(headers)
            .build()
            .map_err(|error| DeSecError::ClientBuilder(error.to_string()))?;
        Ok(DeSecClient { client, api_url: API_URL.into(), token })
    }

    pub async fn get_account_info(&self) -> Result<AccountInformation, DeSecError> {
        match self.get("/auth/account/").await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse account info: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get account information"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get account info"),
                    error
                ))
            }
        }
    }

    pub async fn create_domain(&self, domain: String) -> Result<Domain, DeSecError> {
        match self.post(
            "/domains/",
            format!("{{\"name\": \"{}\"}}", domain)
        ).await {
            Ok(response) if response.status().as_u16() == 201 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse response after creating doamin: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to create domain"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to create domain"),
                    error
                ))
            }
        }
    }

    pub async fn get_domains(&self) -> Result<DomainList, DeSecError> {
        match self.get("/domains/").await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse domain records: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get domains"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get domains"),
                    error
                ))
            }
        }
    }

    pub async fn get_domain(&self, domain: &str) -> Result<Domain, DeSecError> {
        match self.get(format!("/domains/{}/", domain).as_str()).await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse domain record: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get domain"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get domain"),
                    error
                ))
            }
        }
    }

    pub async fn delete_domain(&self, domain: &str) -> Result<String, DeSecError> {
        match self.delete(format!("/domains/{}/", domain).as_str()).await {
            Ok(response) if response.status().as_u16() == 204 => {
                response.text().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse response after deleting domain: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to delete domain"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to delete domain"),
                    error
                ))
            }
        }
    }

    pub async fn get_zonefile(&self, domain: &str) -> Result<String, DeSecError> {
        match self.get(format!(
            "/domains/{}/zonefile/",
            domain
        ).as_str()).await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.text().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to read zonefile response: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get zonefile"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get zonefile"),
                    error
                ))
            }
        }
    }

    pub async fn create_rrset(
        &self, domain: String, subname: String,
        rrset_type: String, records: Vec<String>, ttl: u64)
    -> Result<ResourceRecordSet, DeSecError> {

        let rrset = ResourceRecordSet {
            domain: Some(domain.clone()),
            subname: Some(subname),
            rrset_type: Some(rrset_type),
            records: Some(records),
            ttl: Some(ttl),
            ..ResourceRecordSet::default()
        };
        match self.post(
            format!("/domains/{}/rrsets/", domain).as_str(),
            serde_json::to_string(&rrset).map_err(|err| DeSecError::Parser(err.to_string()))?
        ).await {
            Ok(response) if response.status().as_u16() == 201 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse response after creating rrset: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to create rrset"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to create rrset"),
                    error
                ))
            }
        }
    }

    pub async fn get_rrsets(&self, domain: &str) -> Result<ResourceRecordSetList, DeSecError> {
        match self.get(format!("/domains/{}/rrsets/", domain).as_str()).await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse rrsets: {}", error)
                ))
            },
            Ok(response) if response.status().as_u16() == 404 => {
                Err(DeSecError::NotFound(
                    format!("rrsets for domain {} not found", domain)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get rrsets"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get rrsets"),
                    error
                ))
            }
        }
    }

    pub async fn get_rrset(&self, domain: &str, subname: &str, rrset_type: &str)
    -> Result<ResourceRecordSet, DeSecError> {

        // https://desec.readthedocs.io/en/latest/dns/rrsets.html#accessing-the-zone-apex
        let subname = if subname.is_empty() { "@" } else { subname };

        match self.get(format!(
            "/domains/{}/rrsets/{}/{}/",
            domain, subname, rrset_type
        ).as_str()).await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse rrset: {}", error)
                ))
            },
            Ok(response) if response.status().as_u16() == 404 => {
                Err(DeSecError::NotFound(
                    format!("rrset {}.{} ({}) not found", subname, domain, rrset_type)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to get rrset"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to get rrset"),
                    error
                ))
            }
        }
    }

    pub async fn update_rrset(&self, domain: &str, subname: &str,
        rrset_type: &str, patch: &ResourceRecordSet)
    -> Result<ResourceRecordSet, DeSecError> {

        // https://desec.readthedocs.io/en/latest/dns/rrsets.html#accessing-the-zone-apex
        let subname = if subname.is_empty() { "@" } else { subname };

        match self.patch(
            format!(
                "/domains/{}/rrsets/{}/{}/"
                , domain, subname, rrset_type).as_str(),
            serde_json::to_string(patch).map_err(|err| DeSecError::Parser(err.to_string()))?
        ).await {
            Ok(response) if response.status().as_u16() == 200 => {
                response.json().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse response after updating rrset: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to update rrset"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to update rrset"),
                    error
                ))
            }
        }
    }

    pub async fn delete_rrset(&self, domain: &str, subname: &str, rrset_type: &str)
    -> Result<String, DeSecError> {

        // https://desec.readthedocs.io/en/latest/dns/rrsets.html#accessing-the-zone-apex
        let subname = if subname.is_empty() { "@" } else { subname };

        match self.delete(
            format!(
                "/domains/{}/rrsets/{}/{}/"
                , domain, subname, rrset_type
            ).as_str()
        ).await {
            Ok(response) if response.status().as_u16() == 204 => {
                response.text().await.map_err(|error| DeSecError::Parser(
                    format!("Failed to parse response after deleting rrset: {}", error)
                ))
            },
            Ok(response) => {
                Err(DeSecError::Http(
                    String::from("Failed to delete rrset"),
                    response.error_for_status().err().unwrap()
                ))
            },
            Err(error) => {
                Err(DeSecError::Http(
                    String::from("Failed to delete rrset"),
                    error
                ))
            }
        }
    }

    async fn get(&self, endpoint: &str) -> Result<Response, Error> {
        self.client.get(format!("{}{}", self.api_url, endpoint))
            .send()
            .await
    }

    async fn post(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client.post(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await
    }

    async fn patch(&self, endpoint: &str, body: String) -> Result<Response, Error> {
        self.client.patch(format!("{}{}", self.api_url, endpoint).as_str())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
    }

    async fn delete(&self, endpoint: &str) -> Result<Response, Error> {
        self.client.delete(format!("{}{}", self.api_url, endpoint).as_str())
            .send()
            .await
    }
}
