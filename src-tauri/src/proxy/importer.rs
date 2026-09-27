use crate::proxy::recorder::HeaderEntry;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SavedTemplateInput {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub method: Option<String>,
    pub uri: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub requires_auth: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavedTemplateOutput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tag: Option<String>,
    pub method: String,
    pub uri: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub requires_auth: bool,
}

/// Parser inteligente e universal de coleções (OpenAPI 3.0 / Swagger, Postman Collection v2.1 ou Array Relay)
pub fn parse_collection_content(json_content: &str) -> Result<Vec<SavedTemplateOutput>, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json_content).map_err(|e| format!("JSON inválido: {}", e))?;

    let mut result = Vec::new();

    // 1. Suporte Nativo a OpenAPI 3.0 / Swagger (openapi: "3.0.x" ou swagger: "2.0")
    if let Some(paths) = parsed.get("paths").and_then(|p| p.as_object()) {
        let mut idx = 1;
        for (path_key, path_item) in paths {
            if let Some(methods_map) = path_item.as_object() {
                for (method_key, op_val) in methods_map {
                    let method_upper = method_key.to_uppercase();
                    if !["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]
                        .contains(&method_upper.as_str())
                    {
                        continue;
                    }

                    let summary = op_val
                        .get("summary")
                        .and_then(|s| s.as_str())
                        .or_else(|| op_val.get("operationId").and_then(|o| o.as_str()))
                        .unwrap_or(path_key.as_str())
                        .to_string();

                    let description = op_val
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());

                    let tag = op_val
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|first_tag| first_tag.as_str())
                        .map(|s| s.to_string());

                    let headers = vec![HeaderEntry {
                        key: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    }];

                    let mut body = None;
                    if let Some(req_body) = op_val.get("requestBody") {
                        if let Some(content) = req_body.get("content") {
                            if let Some(json_content) = content.get("application/json") {
                                if let Some(schema) = json_content.get("schema") {
                                    if let Some(example) = json_content
                                        .get("example")
                                        .or_else(|| schema.get("example"))
                                    {
                                        body = Some(
                                            serde_json::to_string_pretty(example)
                                                .unwrap_or_default(),
                                        );
                                    } else {
                                        body = Some("{\n  \"example\": \"data\"\n}".to_string());
                                    }
                                }
                            }
                        }
                    }

                    let requires_auth =
                        op_val.get("security").is_some() || parsed.get("security").is_some();

                    result.push(SavedTemplateOutput {
                        id: format!("openapi-{}", idx),
                        name: summary,
                        description,
                        tag,
                        method: method_upper,
                        uri: path_key.clone(),
                        headers,
                        body,
                        requires_auth,
                    });
                    idx += 1;
                }
            }
        }
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // 2. Suporte Nativo a Postman Collection v2.1
    if let Some(items) = parsed.get("item").and_then(|i| i.as_array()) {
        fn parse_postman_items(
            items: &[serde_json::Value],
            current_tag: Option<String>,
            out: &mut Vec<SavedTemplateOutput>,
            counter: &mut usize,
        ) {
            for it in items {
                // Caso seja uma pasta/grupo recursivo
                if let Some(sub_items) = it.get("item").and_then(|i| i.as_array()) {
                    let folder_name = it
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Pasta")
                        .to_string();
                    parse_postman_items(sub_items, Some(folder_name), out, counter);
                    continue;
                }

                // Caso seja uma requisição individual
                if let Some(req) = it.get("request") {
                    *counter += 1;
                    let name = it
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Requisição Postman")
                        .to_string();

                    let method = req
                        .get("method")
                        .and_then(|m| m.as_str())
                        .unwrap_or("GET")
                        .to_uppercase();

                    let mut uri = String::new();
                    if let Some(url_val) = req.get("url") {
                        if let Some(raw_str) = url_val.as_str() {
                            uri = extract_uri_from_raw(raw_str);
                        } else if let Some(url_obj) = url_val.as_object() {
                            if let Some(raw) = url_obj.get("raw").and_then(|r| r.as_str()) {
                                uri = extract_uri_from_raw(raw);
                            } else if let Some(path_arr) =
                                url_obj.get("path").and_then(|p| p.as_array())
                            {
                                let path_segments: Vec<&str> =
                                    path_arr.iter().filter_map(|s| s.as_str()).collect();
                                if !path_segments.is_empty() {
                                    uri = format!("/{}", path_segments.join("/"));
                                }
                            }
                        }
                    }

                    if uri.is_empty() {
                        uri = "/".to_string();
                    }

                    let mut headers = Vec::new();
                    if let Some(h_arr) = req.get("header").and_then(|h| h.as_array()) {
                        for h in h_arr {
                            if let (Some(k), Some(v)) = (
                                h.get("key").and_then(|k| k.as_str()),
                                h.get("value").and_then(|v| v.as_str()),
                            ) {
                                headers.push(HeaderEntry {
                                    key: k.to_string(),
                                    value: v.to_string(),
                                });
                            }
                        }
                    }

                    let body = req
                        .get("body")
                        .and_then(|b| b.get("raw"))
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string());

                    let requires_auth = req.get("auth").is_some()
                        || headers
                            .iter()
                            .any(|h| h.key.eq_ignore_ascii_case("authorization"));

                    out.push(SavedTemplateOutput {
                        id: format!("postman-{}", counter),
                        name,
                        description: it
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        tag: current_tag.clone(),
                        method,
                        uri,
                        headers,
                        body,
                        requires_auth,
                    });
                }
            }
        }

        fn extract_uri_from_raw(raw: &str) -> String {
            let without_var = if raw.starts_with("{{") {
                if let Some(end_idx) = raw.find("}}") {
                    &raw[end_idx + 2..]
                } else {
                    raw
                }
            } else {
                raw
            };

            if without_var.starts_with('/') {
                without_var.to_string()
            } else if let Some(pos) = without_var.find("://") {
                if let Some(slash_pos) = without_var[pos + 3..].find('/') {
                    without_var[pos + 3 + slash_pos..].to_string()
                } else {
                    "/".to_string()
                }
            } else if let Some(slash_pos) = without_var.find('/') {
                without_var[slash_pos..].to_string()
            } else if !without_var.is_empty() {
                format!("/{}", without_var)
            } else {
                "/".to_string()
            }
        }

        let mut counter = 0;
        parse_postman_items(items, None, &mut result, &mut counter);
        if !result.is_empty() {
            return Ok(result);
        }
    }

    // 3. Array Padrão Relay / JSON Universal
    let raw_array = if let Some(arr) = parsed.as_array() {
        Some(arr)
    } else if let Some(arr) = parsed.get("requests").and_then(|r| r.as_array()) {
        Some(arr)
    } else if let Some(arr) = parsed.get("endpoints").and_then(|r| r.as_array()) {
        Some(arr)
    } else {
        None
    };

    if let Some(arr) = raw_array {
        for (i, item) in arr.iter().enumerate() {
            if let Ok(tpl) = serde_json::from_value::<SavedTemplateInput>(item.clone()) {
                let id = tpl.id.unwrap_or_else(|| format!("tpl-{}", i + 1));
                let uri = tpl
                    .uri
                    .or(tpl.url)
                    .or(tpl.path)
                    .or(tpl.endpoint)
                    .unwrap_or_else(|| "/".to_string());
                let method = tpl.method.unwrap_or_else(|| "GET".to_string()).to_uppercase();
                let name = tpl.name.unwrap_or_else(|| format!("{} {}", method, uri));

                result.push(SavedTemplateOutput {
                    id,
                    name,
                    description: tpl.description,
                    tag: tpl.tag,
                    method,
                    uri,
                    headers: tpl.headers,
                    body: tpl.body,
                    requires_auth: tpl.requires_auth,
                });
            }
        }
        if !result.is_empty() {
            return Ok(result);
        }
    }

    Err("Formato de coleção não reconhecido. Formatos suportados: OpenAPI 3.0 / Swagger JSON, Postman Collection v2.1 ou JSON Array Relay.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relay_array_json() {
        let json = r#"[
            {
                "name": "Listar Usuários",
                "method": "GET",
                "uri": "/api/users",
                "tag": "Users"
            }
        ]"#;

        let res = parse_collection_content(json).expect("Deve parsear array Relay");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Listar Usuários");
        assert_eq!(res[0].method, "GET");
        assert_eq!(res[0].uri, "/api/users");
        assert_eq!(res[0].tag, Some("Users".to_string()));
    }

    #[test]
    fn test_parse_openapi_json() {
        let json = r#"{
            "openapi": "3.0.0",
            "paths": {
                "/api/auth/login": {
                    "post": {
                        "summary": "Login Endpoint",
                        "tags": ["Auth"]
                    }
                }
            }
        }"#;

        let res = parse_collection_content(json).expect("Deve parsear OpenAPI");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Login Endpoint");
        assert_eq!(res[0].method, "POST");
        assert_eq!(res[0].uri, "/api/auth/login");
        assert_eq!(res[0].tag, Some("Auth".to_string()));
    }
}
