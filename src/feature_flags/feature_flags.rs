use chrono::{DateTime, Utc};
use tokio_postgres::Client;

pub struct FeatureFlag {
    pub feature: String,
    pub enabled: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
}

pub struct FeatureFlagController<'r> {
    pub feature_flags: Vec<FeatureFlag>,
    client: &'r Client,
}

impl<'r> FeatureFlagController<'r> {
    pub async fn new(client: &'r Client) -> FeatureFlagController<'r> {
        let mut new_struct = Self {
            feature_flags: Vec::new(),
            client,
        };

        new_struct.get_feature_flags().await;

        new_struct
    }

    pub fn get_feature_flag(&self, feature: &str) -> Option<&FeatureFlag> {
        self.feature_flags.iter().find(|&x| x.feature == feature)
    }

    async fn get_feature_flags(&mut self) {
        let rows = self
            .client
            .query("SELECT * FROM feature_flags", &[])
            .await
            .unwrap();

        for row in rows {
            self.feature_flags.push(FeatureFlag {
                feature: row.get("feature"),
                enabled: row.get("enabled"),
                updated_at: row.get("update_at"),
                updated_by: row.get("updated_by"),
            });
        }
    }
}
