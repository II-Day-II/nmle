// David Kaméus - a tool for visualizing matrix transformations
use nmle::run;

fn main() -> anyhow::Result<()> {
    pollster::block_on(run())
}
