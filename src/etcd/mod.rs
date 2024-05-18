use testcontainers::{core::WaitFor, Image, ImageArgs};

const NAME: &str = "quay.io/coreos/etcd";
const TAG: &str = "v3.5.13";

const ETCD_PORT: u16 = 2379;

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

#[derive(Debug)]
pub struct Etcd;

impl Image for Etcd {
    type Args = EtcdArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
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
    async fn etcd_put_get() {
        let node = etcd::Etcd.start().await;
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
