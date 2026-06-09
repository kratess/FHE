use crate::config::AGGREGATORS_NUM;
use openfhe_bgv_rs::{BgvContext, PublicKey, Result, SecretKey};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct RuntimeLayout {
    root: PathBuf,
}

impl RuntimeLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn setup_dir(&self) -> PathBuf {
        self.root.join("setup")
    }

    pub fn client_dir(&self) -> PathBuf {
        self.root.join("client")
    }

    pub fn aggregator_input_dir(&self, aggregator_idx: usize) -> PathBuf {
        self.client_dir().join(format!("aggregator_{aggregator_idx}"))
    }

    pub fn aggregator_dir(&self) -> PathBuf {
        self.root.join("aggregator")
    }

    pub fn collector_dir(&self) -> PathBuf {
        self.root.join("collector")
    }

    pub fn context_path(&self) -> PathBuf {
        self.setup_dir().join("context.bin")
    }

    pub fn public_key_path(&self) -> PathBuf {
        self.setup_dir().join("public_key.bin")
    }

    pub fn secret_key_path(&self) -> PathBuf {
        self.setup_dir().join("secret_key.bin")
    }

    pub fn eval_mult_key_path(&self) -> PathBuf {
        self.setup_dir().join("eval_mult_key.bin")
    }

    pub fn eval_sum_key_path(&self) -> PathBuf {
        self.setup_dir().join("eval_sum_key.bin")
    }

    pub fn eval_rotate_key_path(&self) -> PathBuf {
        self.setup_dir().join("eval_rotate_key.bin")
    }

    pub fn shard_path(
        &self,
        aggregator_idx: usize,
        client_idx: usize,
        report_idx: usize,
    ) -> PathBuf {
        self.aggregator_input_dir(aggregator_idx)
            .join(format!("client_{client_idx}_report_{report_idx}.bin"))
    }

    pub fn aggregate_share_path(&self, aggregator_idx: usize) -> PathBuf {
        self.aggregator_dir()
            .join(format!("aggregate_share_{aggregator_idx}.bin"))
    }

    pub fn collector_output_path(&self) -> PathBuf {
        self.collector_dir().join("result.txt")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.setup_dir())?;
        fs::create_dir_all(self.client_dir())?;
        fs::create_dir_all(self.aggregator_dir())?;
        fs::create_dir_all(self.collector_dir())?;
        for aggregator_idx in 0..AGGREGATORS_NUM {
            fs::create_dir_all(self.aggregator_input_dir(aggregator_idx))?;
        }
        Ok(())
    }

    pub fn load_client_material(&self) -> Result<(BgvContext, PublicKey)> {
        let ctx = BgvContext::load_from_file(self.context_path())?;
        let pk = PublicKey::load_from_file(&ctx, self.public_key_path())?;
        Ok((ctx, pk))
    }

    pub fn load_aggregator_context(&self) -> Result<BgvContext> {
        let ctx = BgvContext::load_from_file(self.context_path())?;
        ctx.load_eval_mult_key_from_file(self.eval_mult_key_path())?;
        ctx.load_eval_sum_key_from_file(self.eval_sum_key_path())?;
        ctx.load_eval_rotate_key_from_file(self.eval_rotate_key_path())?;
        Ok(ctx)
    }

    pub fn load_collector_material(&self) -> Result<(BgvContext, SecretKey)> {
        let ctx = BgvContext::load_from_file(self.context_path())?;
        let sk = SecretKey::load_from_file(&ctx, self.secret_key_path())?;
        Ok((ctx, sk))
    }
}

pub fn sorted_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}
