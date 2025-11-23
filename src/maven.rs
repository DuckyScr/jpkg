use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

const MAVEN_CENTRAL: &str = "https://repo1.maven.org/maven2";

pub struct MavenClient {
    client: Client,
    base_url: String,
}

impl MavenClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: MAVEN_CENTRAL.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn get_metadata(&self, group_id: &str, artifact_id: &str) -> Result<MavenMetadata> {
        let url = format!(
            "{}/{}/{}/maven-metadata.xml",
            self.base_url,
            group_id.replace('.', "/"),
            artifact_id
        );

        let response = self.client.get(&url).send()?.text()?;
        let metadata: MavenMetadata = quick_xml::de::from_str(&response)?;
        Ok(metadata)
    }

    pub fn get_pom(&self, group_id: &str, artifact_id: &str, version: &str) -> Result<Project> {
        let url = format!(
            "{}/{}/{}/{}/{}-{}.pom",
            self.base_url,
            group_id.replace('.', "/"),
            artifact_id,
            version,
            artifact_id,
            version
        );
        let response = self.client.get(&url).send()?.text()?;
        let project: Project = quick_xml::de::from_str(&response)?;
        Ok(project)
    }

    pub fn download_jar(
        &self,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        output_path: &std::path::Path,
    ) -> Result<()> {
        let url = format!(
            "{}/{}/{}/{}/{}-{}.jar",
            self.base_url,
            group_id.replace('.', "/"),
            artifact_id,
            version,
            artifact_id,
            version
        );
        let mut response = self.client.get(&url).send()?;
        let mut file = std::fs::File::create(output_path)?;
        std::io::copy(&mut response, &mut file)?;
        Ok(())
    }

    pub fn search_artifact(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = "https://search.maven.org/solrsearch/select";
        let response = self
            .client
            .get(url)
            .header("User-Agent", "jpkg/0.1.0")
            .query(&[("q", query), ("rows", "20"), ("wt", "json")])
            .send()?;

        if !response.status().is_success() {
            anyhow::bail!("Search failed: {}", response.status());
        }

        let text = response.text()?;
        // println!("Debug response: {}", text); // Uncomment for debugging
        let response: SearchResponse = serde_json::from_str(&text)
            .context(format!("Failed to parse search response: {}", text))?;

        Ok(response.response.docs)
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    response: SearchResponseBody,
}

#[derive(Debug, Deserialize)]
struct SearchResponseBody {
    docs: Vec<SearchResult>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    #[allow(dead_code)]
    pub id: String,
    pub g: String,
    pub a: String,
    pub latest_version: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MavenMetadata {
    pub group_id: String,
    pub artifact_id: String,
    pub versioning: Versioning,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Versioning {
    pub latest: Option<String>,
    pub release: Option<String>,
    pub versions: Versions,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Versions {
    #[serde(rename = "version")]
    pub version: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    #[allow(dead_code)]
    pub group_id: Option<String>,
    #[allow(dead_code)]
    pub artifact_id: String,
    #[allow(dead_code)]
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: Dependencies,
}

#[derive(Debug, Deserialize, Default)]
pub struct Dependencies {
    #[serde(rename = "dependency", default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub scope: Option<String>,
}
