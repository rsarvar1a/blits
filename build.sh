
if [ ${1} == "debug" ]; then
  cargo build
else 
  cargo build -r
fi

cp -t env/ target/${1}/{client,engine}
