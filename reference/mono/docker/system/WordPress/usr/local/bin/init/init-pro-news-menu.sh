#!/bin/bash

# A script for initializing the Pro News Menu Categories menu. Supports
# deleting the old one, but not by default. A normal way to call this
# locally would be:
# docker-compose exec wp bin/init/init-pro-news-menu.sh

# utility functions

function add_item () {
  echo "Adding item $1."
  /srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item add-term pro-news-menu-categories category ${category_id[$1]}
}

function assign_parent () {
  echo "Assigning $1 to parent $2."
  /srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item update ${menu_id[${category_id[$1]}]} --parent-id=${menu_id[${category_id[$2]}]}
}

function update_title () {
  echo "Changing the title of $1 to $2."
  /srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item update ${menu_id[${category_id[$1]}]} --title="$2"
}

function get_kids () {
  echo "KIDS_${category_id[$1]}"
  declare -n parent_array=KIDS_${category_id[$1]}
  for item in "${parent_array[@]}"; do
    echo $item
  done
}

function add_kids () {
  echo "Adding children of $1."
  declare -n parent_array=KIDS_${category_id[$1]}
  for item in "${parent_array[@]}"; do
    add_item $item
  done
}

function parent_kids () {
  echo "Parenting children of $1."
  declare -n parent_array=KIDS_${category_id[$1]}
  for item in "${parent_array[@]}"; do
    assign_parent $item $1
  done
}

# main script

cd /srv/wp/web

if [[ $1 == "--delete" ]]; then
  echo 'Attempting delete of pro-news-menu-categories.'
  /srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu delete pro-news-menu-categories
else
  if /srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item list pro-news-menu-categories >> /dev/null 2> /dev/null ; then
    echo 'pro-news-menu-categories exists. exiting.'
    exit 1
  fi;
fi;

/srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu create "Pro News Menu Categories"

# build category id vars
declare -A category_id

while IFS=, read slug term_id parent_id; do
  category_id[$slug]=$term_id
  declare -n parent_array=KIDS_$parent_id
  parent_array+=($slug)
done <<< "$(/srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" term list category --fields=slug,term_id,parent --format=csv)"

# put items in menu in display order

add_item deals
add_kids deals
add_item policy
add_kids policy
add_item crypto-ecosystems
add_kids crypto-ecosystems
add_item markets
add_kids markets
add_item companies
add_kids companies
add_item nfts-gaming-and-metaverse
add_kids nfts-gaming-and-metaverse

# build menu item id vars

declare -A menu_id

while IFS=, read term_id db_id; do
  menu_id[$term_id]=$db_id
done <<< "$(/srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item list pro-news-menu-categories --fields=object_id,db_id --format=csv)"

# assign item parents

parent_kids deals
parent_kids policy
parent_kids crypto-ecosystems
parent_kids markets
parent_kids companies
parent_kids nfts-gaming-and-metaverse

# tweak some titles

update_title deals "Pro Deals"
update_title policy "Pro Policy"
update_title crypto-ecosystems "Pro Crypto Ecosystems"

echo "Final pro-news-menu-categories:"

/srv/wp/vendor/bin/wp --allow-root --url="localhost:8080" menu item list pro-news-menu-categories --fields=db_id,type,title,link,position,menu_item_parent,object_id
