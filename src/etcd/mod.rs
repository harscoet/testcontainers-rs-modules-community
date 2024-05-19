use testcontainers::{core::WaitFor, Image, ImageArgs};

const DEFAULT_IMAGE_NAME: &str = "gcr.io/etcd-development/etcd"; // etcd uses gcr.io/etcd-development/etcd as a primary container registry, and quay.io/coreos/etcd as secondary. https://github.com/etcd-io/etcd/blob/main/CHANGELOG/CHANGELOG-3.3.md#other-1
const DEFAULT_IMAGE_TAG: &str = "v3.5.13"; // https://github.com/etcd-io/etcd/tags

pub const ETCD_PORT: u16 = 2379;

/// Module to work with [`etcd`] inside of tests.
///
/// This module is based on the official [`etcd docker image`].
///
/// # Example
/// ```
/// use testcontainers_modules::{etcd, testcontainers::runners::SyncRunner};
///
/// let etcd = etcd::Etcd::default().start();
/// let http_port = etcd.get_host_port_ipv4(etcd::ETCD_PORT);
///
/// // do something with the started etcd instance..
/// ```
///
/// [`etcd`]: https://etcd.io/
/// [`etcd configuration`]: https://etcd.io/docs/v3.5/op-guide/configuration/#command-line-flags
/// [`etcd docker image`]: https://gcr.io/etcd-development/etcd
/// [`etcd docker guide`]: https://etcd.io/docs/v3.5/op-guide/container/
#[derive(Debug)]
pub struct Etcd {
    name: String,
    tag: String,
}

impl Default for Etcd {
    fn default() -> Self {
        Self {
            name: DEFAULT_IMAGE_NAME.to_owned(),
            tag: DEFAULT_IMAGE_TAG.to_owned(),
        }
    }
}

impl Etcd {
    pub fn new<T: Into<String>>(tag: T) -> Self {
        Self {
            tag: tag.into(),
            ..Default::default()
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct EtcdArgs;

impl ImageArgs for EtcdArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            vec![
                "etcd".to_owned(),
                "-advertise-client-urls".to_owned(),
                format!("http://127.0.0.1:{ETCD_PORT}"),
                "-listen-client-urls".to_owned(),
                format!("http://0.0.0.0:{ETCD_PORT}"),
            ]
            .into_iter(),
        )
    }
}

impl Image for Etcd {
    type Args = EtcdArgs;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        self.tag.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("ready to serve client requests")]
    }
}

#[cfg(test)]
mod tests {
    use etcd_client::Client;
    use testcontainers::runners::AsyncRunner;

    use crate::etcd;

    #[tokio::test]
    async fn etcd_default() {
        let etcd = etcd::Etcd::default();
        assert_eq!(etcd.name, etcd::DEFAULT_IMAGE_NAME);
        assert_eq!(etcd.tag, etcd::DEFAULT_IMAGE_TAG);

        etcd_put_get(etcd).await;
    }

    #[tokio::test]
    async fn etcd_custom_image_tag() {
        let image_tag: &str = "v3.4.10";

        let etcd = etcd::Etcd::new(image_tag);
        assert_eq!(etcd.name, etcd::DEFAULT_IMAGE_NAME);
        assert_eq!(etcd.tag, image_tag);

        etcd_put_get(etcd).await;
    }

    #[tokio::test]
    async fn etcd_custom_image_name() {
        let image_name: &str = "quay.io/coreos/etcd";

        let etcd = etcd::Etcd::default().name(image_name);
        assert_eq!(etcd.name, image_name);
        assert_eq!(etcd.tag, etcd::DEFAULT_IMAGE_TAG);

        etcd_put_get(etcd).await;
    }

    async fn etcd_put_get(etcd: etcd::Etcd) {
        let node = etcd.start().await;
        let host_ip = node.get_host().await;
        let host_port = node.get_host_port_ipv4(etcd::ETCD_PORT).await;

        let mut client = Client::connect([format!("{host_ip}:{host_port}")], None)
            .await
            .expect("connect failed");

        client.put("foo", "bar", None).await.expect("put failed");

        assert_eq!(
            client
                .get("foo", None)
                .await
                .expect("get failed")
                .kvs()
                .first()
                .expect("no kv found")
                .value_str()
                .unwrap(),
            "bar"
        );
    }
}
