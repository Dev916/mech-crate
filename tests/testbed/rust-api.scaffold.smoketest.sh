
cd workbench

rm -rf new-rust-api

mx new new-rust-api --no-prompt

cd new-rust-api

mx add rust-api --recipe=rust-api --no-prompt

mx build rust-api --prod --no-cache

make dev s=rust-api