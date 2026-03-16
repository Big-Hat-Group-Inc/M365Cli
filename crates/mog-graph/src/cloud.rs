use serde::{Deserialize, Serialize};

/// Supported cloud environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cloud {
    #[default]
    Public,
    Gcc,
    GccHigh,
    Dod,
    China,
}

impl Cloud {
    pub fn endpoints(&self) -> CloudEndpoints {
        match self {
            Cloud::Public | Cloud::Gcc => CloudEndpoints {
                authority: "https://login.microsoftonline.com",
                graph: "https://graph.microsoft.com",
                resource: "https://graph.microsoft.com/.default",
            },
            Cloud::GccHigh => CloudEndpoints {
                authority: "https://login.microsoftonline.us",
                graph: "https://graph.microsoft.us",
                resource: "https://graph.microsoft.us/.default",
            },
            Cloud::Dod => CloudEndpoints {
                authority: "https://login.microsoftonline.us",
                graph: "https://dod-graph.microsoft.us",
                resource: "https://dod-graph.microsoft.us/.default",
            },
            Cloud::China => CloudEndpoints {
                authority: "https://login.chinacloudapi.cn",
                graph: "https://microsoftgraph.chinacloudapi.cn",
                resource: "https://microsoftgraph.chinacloudapi.cn/.default",
            },
        }
    }
}

impl std::fmt::Display for Cloud {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cloud::Public => write!(f, "public"),
            Cloud::Gcc => write!(f, "gcc"),
            Cloud::GccHigh => write!(f, "gcc-high"),
            Cloud::Dod => write!(f, "dod"),
            Cloud::China => write!(f, "china"),
        }
    }
}

impl std::str::FromStr for Cloud {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "public" => Ok(Cloud::Public),
            "gcc" => Ok(Cloud::Gcc),
            "gcc-high" | "gcchigh" => Ok(Cloud::GccHigh),
            "dod" => Ok(Cloud::Dod),
            "china" => Ok(Cloud::China),
            _ => Err(format!("Unknown cloud: {}. Valid: public, gcc, gcc-high, dod, china", s)),
        }
    }
}

/// Endpoints for a cloud environment
#[derive(Debug, Clone)]
pub struct CloudEndpoints {
    pub authority: &'static str,
    pub graph: &'static str,
    pub resource: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_endpoints() {
        let public = Cloud::Public.endpoints();
        assert_eq!(public.authority, "https://login.microsoftonline.com");
        assert_eq!(public.graph, "https://graph.microsoft.com");

        let dod = Cloud::Dod.endpoints();
        assert_eq!(dod.graph, "https://dod-graph.microsoft.us");
    }

    #[test]
    fn test_cloud_from_str() {
        assert_eq!("public".parse::<Cloud>().unwrap(), Cloud::Public);
        assert_eq!("gcc-high".parse::<Cloud>().unwrap(), Cloud::GccHigh);
        assert_eq!("gcchigh".parse::<Cloud>().unwrap(), Cloud::GccHigh);
        assert!("invalid".parse::<Cloud>().is_err());
    }

    #[test]
    fn test_all_cloud_endpoints() {
        let gcc = Cloud::Gcc.endpoints();
        assert_eq!(gcc.graph, "https://graph.microsoft.com"); // Same as public

        let gcc_high = Cloud::GccHigh.endpoints();
        assert_eq!(gcc_high.authority, "https://login.microsoftonline.us");
        assert_eq!(gcc_high.graph, "https://graph.microsoft.us");

        let china = Cloud::China.endpoints();
        assert_eq!(china.authority, "https://login.chinacloudapi.cn");
        assert_eq!(china.graph, "https://microsoftgraph.chinacloudapi.cn");
    }

    #[test]
    fn test_cloud_display() {
        assert_eq!(Cloud::Public.to_string(), "public");
        assert_eq!(Cloud::GccHigh.to_string(), "gcc-high");
        assert_eq!(Cloud::Dod.to_string(), "dod");
        assert_eq!(Cloud::China.to_string(), "china");
    }

    #[test]
    fn test_cloud_default() {
        assert_eq!(Cloud::default(), Cloud::Public);
    }
}
