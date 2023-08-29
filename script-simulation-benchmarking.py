import os
import time

# before running: delete manually databases from goshimmer and wasp
# start goshimmer and wasp manually in separate terminals (see commented out code), this to view feedback messages while running

initial_funding = 10000
total_clients = 100
time_interval = 1
amount_of_runs = 100


"""
os.popen("cd ..; cd goshimmer; ./goshimmer  --autopeering.seed=base58:8q491c3YWjbPwLmF2WD95YmCgh61j2kenCKHfGfByoWi  --node.enablePlugins=bootstrap,prometheus,spammer,"webapi tools endpoint",activity,snapshot,txstream   --messageLayer.startSynced=true   --autopeering.entryNodes=       --node.disablePlugins=clock       --messageLayer.snapshot.file=./assets/snapshotTest.bin       --messageLayer.snapshot.genesisNode=       --metrics.manaResearch=false       --mana.enableResearchVectors=false       --mana.snapshotResetTime=true       --statement.writeStatement=true --statement.writeManaThreshold=1.0 --config=./config.json")
print("Started GoShimmer succesfully")
time.sleep(10)
"""
"""
os.popen("cd ..; cd wasp; ./wasp")
print("Started WASP succesfully")
time.sleep(10)
"""

address_output = os.popen("cd ..; cd wasp; ./wasp-cli address").read()
wasp_address = address_output[31:76]
print("WASP address is: " + wasp_address)
funding_output = os.popen("cd ..; cd wallet; ./cli-wallet send-funds -amount " + str(initial_funding) + " -dest-addr " + wasp_address).read()
print(funding_output)
time.sleep(1)
print("Provided " + str(initial_funding) + " IOTA to: " + wasp_address)


chain_deploy = os.popen("cd ..; cd wasp; ./wasp-cli chain deploy --committee=0 --quorum=1 --chain=energymarketchain --description=\"Energy Market\"").read()
time.sleep(5)
print(chain_deploy)

chain_deposit = os.popen("cd ..; cd wasp; ./wasp-cli chain deposit IOTA:1000 --chain=energymarketchain").read()
time.sleep(5)
print(chain_deposit)



os.popen("cd ..; mkdir wasp-clients")
print("Made directory wasp-clients")
time.sleep(1)

os.popen("cd ..; cd wasp-clients; mkdir wasp-client-0")
print("Made directory wasp-client-0")
time.sleep(1)

os.popen("cd ..; cd wasp; cp wasp-cli wasp-cli.json ~/wasp-clients/wasp-client-0/")
time.sleep(1)
os.popen("cd ..; cd wasp-clients; cd wasp-client-0; ./wasp-cli init")
time.sleep(1)


for i in range(1, total_clients + 1):
	os.popen("cd ..; cd wasp-clients; cp -R wasp-client-0 wasp-client-" + str(i))
	time.sleep(2)
time.sleep(5)
print("Copying clients done")
for i in range(1, total_clients + 1):
	os.popen("cd ..; cd wasp-clients; cd wasp-client-" + str(i) + "; ./wasp-cli init")
	time.sleep(0.1)
time.sleep(5)
print("Initializing clients done")
for i in range(1, total_clients + 1):
	client_output = os.popen("cd ..; cd wasp-clients; cd wasp-client-" + str(i) + "; ./wasp-cli address").read()
	time.sleep(0.1)
	client_address = client_output[31:76]
	time.sleep(0.1)
	print(client_address)
	funding_output = os.popen("cd ..; cd wallet; ./cli-wallet send-funds -amount " + str(initial_funding) + " -dest-addr " + client_address).read()
	print(funding_output)
	time.sleep(0.1)
print("Funding clients done")
	
	
os.popen("cd ..; cd wasp-clients; rm -R wasp-client-0")
time.sleep(0.1)
	

chain_deploy_contract = os.popen("cd ..; cd wasp; ./wasp-cli chain deploy-contract wasmtime energymarket \"Energy Market SC\" ./energy-market-smart-contract/pkg/energymarket_bg.wasm  --chain=energymarketchain --upload-quorum=1 -d --address-index=0").read()
time.sleep(5)
print(chain_deploy_contract)
print("CONTRACT DEPLOYED")

init_market = os.popen("cd ..; cd wasp; ./wasp-cli chain post-request energymarket initmarket --chain=energymarketchain").read()
time.sleep(5)
print(init_market)
print("MARKET INITIATED")

for x in range(1, amount_of_runs + 1):
	for i in range(1, total_clients + 1):{
		os.popen("cd ..; cd wasp-clients; cd wasp-client-" + str(i) + "; ./wasp-cli chain post-request energymarket trade string TRADEVALUE string request string WATT string 5 --chain=energymarketchain -t IOTA:10"),
	},
	print("RUN NUMBER: " + str(x) + " OF A TOTAL OF " + str(amount_of_runs)),
	time.sleep(time_interval)
	
