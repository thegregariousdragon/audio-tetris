use std::process::Command;

fn get_version() -> String {
    if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
    {
        if output.status.success() {
            let git_ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !git_ver.is_empty() {
                let clean_ver = git_ver.strip_prefix('v').unwrap_or(&git_ver);
                return clean_ver.to_string();
            }
        }
    }
    env!("CARGO_PKG_VERSION").to_string()
}

fn main() {
    let version = get_version();
    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
"#);
        let _ = res.compile();
    }
}
