
cd workbench

rm -rf lara

mx new lara --no-prompt

cd lara

mx add vel --recipe=laravel --no-prompt

mx build vel --prod --no-cache

mx dev vel