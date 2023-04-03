use crate::error::Error;
use chrono::DateTime;
use chrono::Utc;
use etcd_client::GetOptions;
use serde::Serialize;
use std::fmt;
use url::Url;

#[derive(Clone)]
pub struct Db {
    prefix: String,
    // conneciton pool is not needed because tonic channel in client will reconect
    // use clone capablity of client to get multiplexed client for each request
    client: etcd_client::Client,
}

impl Db {
    pub fn new(client: etcd_client::Client, prefix: String) -> Self {
        Self { prefix, client }
    }

    fn new_prefixed_property_key<K: Key>(
        &self,
        property: String,
        key: K,
    ) -> PrefixedKey<PropertyKey<K>> {
        PrefixedKey::new(self.prefix.clone(), PropertyKey::new(property, key))
    }

    pub async fn add_endpoint_url(
        &mut self,
        endpoint: EndpointKey,
        url: &url::Url,
    ) -> Result<(), Error> {
        let key = self
            .new_prefixed_property_key("url".into(), endpoint)
            .to_string();
        let value = url.to_string();
        self.client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_endpoint_url(&mut self, endpoint: EndpointKey) -> Result<url::Url, Error> {
        let key = self
            .new_prefixed_property_key("url".into(), endpoint)
            .to_string();
        let res = self.client.get(key, None).await?;
        let url = res
            .kvs()
            .get(0)
            .ok_or(Error::NotFound)?
            .value_str()?
            .parse()?;
        Ok(url)
    }

    pub async fn get_all_endpoint_urls(&mut self) -> Result<Vec<EndpointRecord>, Error> {
        let options = GetOptions::new().with_prefix();
        let key = self
            .new_prefixed_property_key("url".into(), EmptyKey)
            .to_string();
        let res = self.client.get(key, Some(options)).await?;
        res.kvs()
            .into_iter()
            .map(|kv| {
                let path: PrefixedKey<PropertyKey<EndpointKey>> = kv.key_str()?.parse()?;
                let key = path.key.inner.key;
                let url = kv.value_str()?.parse()?;
                Ok(EndpointRecord { key, url })
            })
            .collect()
    }

    // pub async fn get_all_endpoint_urls(&mut self) -> impl Stream<Item = Result<EndpointRecord, Error>> {
    //     use etcd_client::{GetOptions, SortOrder, SortTarget};
    //     let limit: usize = 100;
    //     let (sender, receiver) = mpsc::channel(limit);
    //     let options = GetOptions::new()
    //         .with_prefix()
    //         .with_limit(limit as i64)
    //         .with_sort(SortTarget::Key, SortOrder::Ascend);
    //     let key_range_start = self.prefix.clone();
    //     let res = self.client.get(key_range_start, Some(options)).await;

    //     // TODO: use mpsc::bounded to provide a stream and spawn a task that uses a multi stept get request with pointer
    //     receiver

    // first range
    // # etcdctl -w fields get --keys-only --sort-by=KEY --limit 2 eccer eccer\0
    // following ranges with last key + \0
    // # etcdctl -w fields get --keys-only --sort-by=KEY --rev 4 eccer/url/myservice/1/https\0 eccer\0
    // }

    async fn get_endpoint_property(
        &mut self,
        endpoint: EndpointKey,
        property: String,
    ) -> Result<String, Error> {
        let key = self
            .new_prefixed_property_key(property, endpoint)
            .to_string();
        let res = self.client.get(key, None).await?;
        let s = res
            .kvs()
            .get(0)
            .ok_or(Error::NotFound)?
            .value_str()?
            .to_string();
        Ok(s)
    }

    async fn set_endpoint_property(
        &mut self,
        endpoint: EndpointKey,
        property: String,
        value: String,
    ) -> Result<(), Error> {
        let key = self
            .new_prefixed_property_key(property, endpoint)
            .to_string();
        self.client.put(key, value, None).await?;
        Ok(())
    }

    pub async fn get_endpoint_stats(
        &mut self,
        endpoint: EndpointKey,
    ) -> Result<EndpointStats, Error> {
        let url: Url = self
            .get_endpoint_property(endpoint.clone(), "url".to_string())
            .await?
            .parse()?;
        let last_success: Option<DateTime<Utc>> = match self
            .get_endpoint_property(endpoint.clone(), "last_success".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?;
        let last_failure: Option<DateTime<Utc>> = match self
            .get_endpoint_property(endpoint.clone(), "last_failure".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?;
        let success_count: u64 = match self
            .get_endpoint_property(endpoint.clone(), "success_count".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?
        .unwrap_or(0);
        let failure_count: u64 = match self
            .get_endpoint_property(endpoint.clone(), "failure_count".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?
        .unwrap_or(0);
        Ok(EndpointStats {
            key: endpoint,
            url,
            last_success,
            last_failure,
            success_count,
            failure_count,
        })
    }

    pub async fn record_endpoint_success(
        &mut self,
        endpoint: EndpointKey,
        dt: DateTime<Utc>,
    ) -> Result<u64, Error> {
        self.set_endpoint_property(
            endpoint.clone(),
            "last_success".to_string(),
            dt.to_rfc3339(),
        )
        .await?;
        let success_count: u64 = match self
            .get_endpoint_property(endpoint.clone(), "success_count".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?
        .unwrap_or(0);
        let new_success_count = success_count + 1;
        self.set_endpoint_property(
            endpoint,
            "success_count".to_string(),
            format!("{}", new_success_count),
        )
        .await?;
        Ok(new_success_count)
    }

    pub async fn record_endpoint_failure(
        &mut self,
        endpoint: EndpointKey,
        dt: DateTime<Utc>,
    ) -> Result<u64, Error> {
        self.set_endpoint_property(
            endpoint.clone(),
            "last_failure".to_string(),
            dt.to_rfc3339(),
        )
        .await?;
        let success_count: u64 = match self
            .get_endpoint_property(endpoint.clone(), "failure_count".to_string())
            .await
        {
            Ok(s) => s.parse().map(|d| Some(d)).map_err(From::from),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }?
        .unwrap_or(0);
        let new_success_count = success_count + 1;
        self.set_endpoint_property(
            endpoint,
            "failure_count".to_string(),
            format!("{}", new_success_count),
        )
        .await?;
        Ok(new_success_count)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointRecord {
    pub key: EndpointKey,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointStats {
    pub key: EndpointKey,
    pub url: Url,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub success_count: u64,
    pub failure_count: u64,
}

trait Key: fmt::Display + std::str::FromStr<Err = Error> {}

#[derive(Debug, Clone)]
struct PrefixedKey<K: Key> {
    prefix: String,
    key: K,
}

impl<K: Key> Key for PrefixedKey<K> {}

impl<K: Key> fmt::Display for PrefixedKey<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.prefix, self.key)
    }
}

impl<K: Key> std::str::FromStr for PrefixedKey<K> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, "/");
        let prefix = parts.next().ok_or(Error::ParseEtcdKeyError)?.to_owned();
        let rest = parts.next().ok_or(Error::ParseEtcdKeyError)?;
        let key = rest.parse()?;
        Ok(PrefixedKey { prefix, key })
    }
}

impl<K: Key> PrefixedKey<K> {
    pub fn new(prefix: String, key: K) -> Self {
        PrefixedKey { prefix, key }
    }
}

#[derive(Debug, Clone)]
struct PropertyKey<K: Key> {
    inner: PrefixedKey<K>,
}

impl<K: Key> Key for PropertyKey<K> {}

impl<K: Key> fmt::Display for PropertyKey<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl<K: Key> std::str::FromStr for PropertyKey<K> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PropertyKey { inner: s.parse()? })
    }
}

impl<K: Key> PropertyKey<K> {
    pub fn new(property: String, key: K) -> Self {
        PropertyKey {
            inner: PrefixedKey {
                prefix: property,
                key,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointKey {
    pub service_name: String,
    pub instance_name: String,
    pub endpoint_name: String,
}

impl Key for EndpointKey {}

impl fmt::Display for EndpointKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.service_name, self.instance_name, self.endpoint_name
        )
    }
}

impl std::str::FromStr for EndpointKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, "/");
        let service_name = parts.next().ok_or(Error::ParseEtcdKeyError)?.to_owned();
        let instance_name = parts.next().ok_or(Error::ParseEtcdKeyError)?.to_owned();
        let endpoint_name = parts.next().ok_or(Error::ParseEtcdKeyError)?.to_owned();
        Ok(EndpointKey {
            service_name,
            instance_name,
            endpoint_name,
        })
    }
}

#[derive(Debug, Clone)]
struct EmptyKey;

impl Key for EmptyKey {}

impl fmt::Display for EmptyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl std::str::FromStr for EmptyKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s != "" {
            return Err(Error::ParseEtcdKeyError);
        }
        Ok(EmptyKey)
    }
}
