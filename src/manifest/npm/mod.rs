mod commonjs;
pub mod repository;

pub use self::commonjs::CommonJSPackage;

#[derive(Serialize)]
#[serde(untagged)]
pub enum NpmPackage {
    CommonJSPackage(CommonJSPackage),
}
