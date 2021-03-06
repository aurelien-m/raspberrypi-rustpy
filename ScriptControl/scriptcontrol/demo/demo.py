from gpiozero import LED
from time import sleep
from random import randint

import rpipy # custom rust written library for python
import luminolib # python objects for the leds and the settings

param = luminolib.Settings() # settings you want to be able to modify on the webserver
param.my_var = 50
param.my_message = "Hello!"
param.slider_var = luminolib.Settings.SliderValue(0, 100, 50)
param.another_slider = luminolib.Settings.SliderValue(10, 40, 20)

led_matrix = luminolib.Led(64) # the leds you can control
turned_on_light = 0

def start():
    print("Device info:", rpipy.get_device_info(), "| temp:", rpipy.measure_temp())

def loop():
    print("Looping in demo! My var =", param.my_var, "| My message:", param.my_message, "| My slider value:", param.slider_var.value)
    global turned_on_light

    led = led_matrix.get(turned_on_light)
    led.green = 255
    led.red = 255
    led.blue = 255

    if turned_on_light > 0:
        led = led_matrix.get(turned_on_light - 1)
        led.green = 0
        led.red = 255
        led.blue = 0

    turned_on_light += 1
    if turned_on_light > 63:
        turned_on_light = 0

    # sleep(5)
    param.my_var += 1
    print("Ending loop in demo!")

def end():
    print("Ending demo!")

def my_func():
    print("Some custom stuff!")