#!/bin/bash 
convert battery-0.bmp -type palette -colors 8 battery-0.bmp
convert battery-20.bmp -type palette -colors 8 battery-20.bmp
convert battery-40.bmp -type palette -colors 8 battery-40.bmp
convert battery-60.bmp -type palette -colors 8 battery-60.bmp
convert battery-80.bmp -type palette -colors 8 battery-80.bmp
convert battery-100.bmp -type palette -colors 8 battery-100.bmp

convert n0.bmp -type grayscale -colors 2 n0.bmp 
convert n1.bmp -type grayscale -colors 2 n1.bmp 
convert n2.bmp -type grayscale -colors 2 n2.bmp 
convert n3.bmp -type grayscale -colors 2 n3.bmp 
convert n4.bmp -type grayscale -colors 2 n4.bmp 
convert n5.bmp -type grayscale -colors 2 n5.bmp 
convert n6.bmp -type grayscale -colors 2 n6.bmp 
convert n7.bmp -type grayscale -colors 2 n7.bmp 
convert n8.bmp -type grayscale -colors 2 n8.bmp 
convert n9.bmp -type grayscale -colors 2 n9.bmp 
convert c.bmp -type grayscale -colors 2 c.bmp
convert dot.bmp -type grayscale -colors 2 dot.bmp
convert minus.bmp -type grayscale -colors 2 minus.bmp
convert usb-power.bmp -type palette -colors 8 usb-power.bmp
convert v.bmp -type palette -colors 8 v.bmp