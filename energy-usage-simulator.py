import time
import datetime
from datetime import date
import random

while True:

    # Read the time
    now = datetime.datetime.now().time()
    
    # Devices, with their energy (power) usage expressed in Watts
    fridge = -100
    freezer = -250
    clocks = -20
    standby_devices = -15
    lights = -random.randrange(0, 150)
    
    total_energy = fridge + freezer + clocks + standby_devices + lights
    
    hour = now.hour
    
    # Depending on the daytime, different base energy usage
    if hour >= 8 and hour < 12:
        total_energy = total_energy + 800
    
    if hour >= 12 and hour < 16:
        total_energy = total_energy + 1200
    
    if hour >= 16 and hour < 18:
        total_energy = total_energy + 400
    
    # Random chance
    random_chance = random.randrange(0, 100)

    # Peak usage: microwave
    if random_chance <= 5:
        total_energy = total_energy - 1000
        
    # Peak usage: fryer
    if random_chance <= 2:
        total_energy = total_energy - 2000
    
    print(str(now) + " the energy usage is " + str(total_energy) + " Watt")
    time.sleep(1)