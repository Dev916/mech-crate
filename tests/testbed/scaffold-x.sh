
cd workbench

PROJECT_NAME=${1:-amazing_app_$(date +%Y%m%d)}

RECIPE_NAME=${2:-rust-api}

rm -rf $PROJECT_NAME

mx new $PROJECT_NAME --no-prompt

cd $PROJECT_NAME

mx add $RECIPE_NAME --recipe=$RECIPE_NAME --no-prompt

mx build $RECIPE_NAME --prod --no-cache

make dev s=$RECIPE_NAME