#! /bin/sh

usage="Usage: install-telegram-bot-api.sh <absolute path to telegram-bot-api> <absolute path to environment file>"

if [ -z "$1" ]
  then
    echo ${usage}
    exit 1
fi

if [ -z "$2" ]
  then
    echo ${usage}
    exit 1
fi

wdir=${HOME}/.local/share/telegram-bot-api

mkdir -p ${wdir}

echo "============== Creating systemd service =============="

echo \
"[Unit]
Description=Service for local Telegram Bot API
After=network.target

[Service]
User=$(whoami)
EnvironmentFile=$2
ExecStart=$1 --local --dir=${wdir}

[Install]
WantedBy=multi-user.target"  > telegram-bot-api.service

echo -e "================= telegram-bot-api.service ==================\n"
cat telegram-bot-api.service
echo -e "\n=============== end telegram-bot-api.service ================"

echo -e "Copying Unit file shown above into /etc/systemd/system/ for service setup"
sudo systemctl stop telegram-bot-api -q
sudo cp telegram-bot-api.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable telegram-bot-api
sudo systemctl start telegram-bot-api
