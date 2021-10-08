fn main() -> anyhow::Result<()> {
    tries::run(tries::Significance::Five)
}
