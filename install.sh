#! /bin/sh

echo "============== Creating systemd service =============="

wdir=$(pwd)

echo \
"[Unit]
Description = Rust DLE bot.
After=network.target

[Service]
User=$(whoami)
WorkingDirectory=${wdir}
ExecStart=${wdir}/target/release/dlebot

[Install]
WantedBy=multi-user.target"  > dlebot.service

echo -e "================= dlebot.service ==================\n"
cat dlebot.service
echo -e "\n=============== end dlebot.service ================"

echo -e "Copying Unit file shown above into /etc/systemd/system/ for service setup"
sudo systemctl stop dlebot -q
sudo cp dlebot.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable dlebot
sudo systemctl start dlebot
