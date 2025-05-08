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
};

use super::{
    OutputKind,
    Tool,
    InvokeOutput,
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

impl UseAws {
    pub async fn invoke(&self, ctx: &Context) -> Result<OutputKind> {
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

        // Use String::from_utf8_lossy instead of ByteSlice::to_str_lossy for Windows compatibility
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            error!(?cmd, %stderr, "AWS CLI command failed");
            bail!("AWS CLI command failed: {}", stderr);
        }

        debug!(?cmd, %stdout, "AWS CLI command succeeded");

        // Parse the output as JSON
        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(json_value) => Ok(OutputKind::Json(json_value)),
            Err(_) => Ok(OutputKind::Json(serde_json::Value::String(stdout.to_string())))
        }
    }

    pub async fn validate(&mut self, _ctx: &Context) -> Result<()> {
        // Validation logic here
        Ok(())
    }

    pub fn requires_acceptance(&self) -> bool {
        true
    }

    pub fn queue_description(&self, _updates: &mut impl std::io::Write) -> Result<()> {
        // Description logic here
        Ok(())
    }
}

impl super::ToolImpl for UseAws {
    type Output = OutputKind;

    async fn invoke(&self, ctx: &Context, _updates: &mut impl std::io::Write) -> Result<InvokeOutput> {
        let output = self.invoke(ctx).await?;
        Ok(InvokeOutput { output })
    }

    async fn validate(&mut self, ctx: &Context) -> Result<()> {
        self.validate(ctx).await
    }

    fn queue_description(&self, updates: &mut impl std::io::Write) -> Result<()> {
        self.queue_description(updates)
    }
}
