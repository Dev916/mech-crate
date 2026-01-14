
cd workbench

rm -rf lara

mx new lara --no-prompt

cd lara

mx add vel --recipe=laravel --no-prompt

make build s=vel BUILD_MODE=prod