use std::str;

use bstr::ByteSlice;
use convert_case::{
    Case,
    Casing,
};
use eyre::{
    bail,
    Result,
};
use fig_os_shim::Context;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::process::Command;
use tracing::{
    debug,
    error,
    warn,
};

use super::{
    OutputKind,
    Tool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseAws {
    pub label: String,
    pub service_name: String,
    pub operation_name: String,
    pub parameters: serde_json::Value,
    pub region: String,
    pub profile_name: Option<String>,
}

impl Tool for UseAws {
    type Output = OutputKind;

    async fn invoke(&self, ctx: &Context) -> Result<Self::Output> {
        let mut cmd = Command::new("aws");
        cmd.arg(self.service_name.as_str())
            .arg(self.operation_name.as_str())
            .arg("--region")
            .arg(self.region.as_str());

        if let Some(profile_name) = &self.profile_name {
            cmd.arg("--profile").arg(profile_name.as_str());
        }

        if let Some(obj) = self.parameters.as_object() {
            for (k, v) in obj {
                let param_name = format!("--{}", k.to_case(Case::Kebab));
                if v.is_null() || (v.is_string() && v.as_str().unwrap().is_empty()) {
                    cmd.arg(param_name);
                } else if v.is_string() {
                    cmd.arg(param_name).arg(v.as_str().unwrap());
                } else {
                    cmd.arg(param_name).arg(v.to_string());
                }
            }
        }

        debug!(?cmd, "Running AWS CLI command");

        let output = cmd.output().await?;

        #[cfg(unix)]
        {
            let stdout = output.stdout.to_str_lossy();
            let stderr = output.stderr.to_str_lossy();

            if !output.status.success() {
                error!(?cmd, %stderr, "AWS CLI command failed");
                bail!("AWS CLI command failed: {}", stderr);
            }

            debug!(?cmd, %stdout, "AWS CLI command succeeded");

            Ok(OutputKind::Json(stdout.to_string()))
        }

        #[cfg(not(unix))]
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                error!(?cmd, %stderr, "AWS CLI command failed");
                bail!("AWS CLI command failed: {}", stderr);
            }

            debug!(?cmd, %stdout, "AWS CLI command succeeded");

            Ok(OutputKind::Json(stdout.to_string()))
        }
    }
}
