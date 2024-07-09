#!/usr/bin/python3
import cv2
import numpy as np
import sys


print(f"opening {sys.argv[1]}")
cap = cv2.VideoCapture(sys.argv[1])
got_frame, frame = cap.read()

def nothing(x):
    pass
# Creating a window for later use
cv2.namedWindow('result')

# Starting with 100's to prevent error while masking
h,s,v = 100,100,100

# Creating track bar
cv2.createTrackbar('h hi', 'result',179,179,nothing)
cv2.createTrackbar('h lo', 'result',0,179,nothing)
cv2.createTrackbar('s hi', 'result',255,255,nothing)
cv2.createTrackbar('s lo', 'result',0,255,nothing)
cv2.createTrackbar('v hi', 'result',255,255,nothing)
cv2.createTrackbar('v lo', 'result',0,255,nothing)

paused = False
loop_num = 0
is_picture = False
while(1):
    if not paused and not is_picture:
        got_frame, frame = cap.read()
        if not got_frame:
            cap.set(cv2.CAP_PROP_POS_FRAMES, 0)
            print(f"looped {loop_num}")
            loop_num += 1
            # _, frame = cap.read()
            continue

    #converting to HSV
    hsv = cv2.cvtColor(frame,cv2.COLOR_BGR2HSV)

    # get info from track bar and appy to result
    h_hi = cv2.getTrackbarPos('h hi','result')
    h_lo = cv2.getTrackbarPos('h lo','result')
    s_hi = cv2.getTrackbarPos('s hi','result')
    s_lo = cv2.getTrackbarPos('s lo','result')
    v_hi = cv2.getTrackbarPos('v hi','result')
    v_lo = cv2.getTrackbarPos('v lo','result')

    # Normal masking algorithm
    lower_blue = np.array([h_lo, s_lo, v_lo])
    upper_blue = np.array([h_hi, s_hi, v_hi])

    mask = cv2.inRange(hsv,lower_blue, upper_blue)
    cv2.imshow('mask',mask)

    result = cv2.bitwise_and(frame,frame,mask = mask)
    cv2.rectangle(result,(200,360),(440,480),(0,255,0),3)
    cv2.imshow('result',result)

    cv2.imshow('orig', frame)

    k = cv2.waitKey(5) & 0xFF
    if k == 32:
        paused = not paused
    elif k != 255:
        break

h_hi = cv2.getTrackbarPos('h hi','result')
h_lo = cv2.getTrackbarPos('h lo','result')
s_hi = cv2.getTrackbarPos('s hi','result')
s_lo = cv2.getTrackbarPos('s lo','result')
v_hi = cv2.getTrackbarPos('v hi','result')
v_lo = cv2.getTrackbarPos('v lo','result')
print(f"({h_lo}, {s_lo}, {v_lo}), ({h_hi}, {s_hi}, {v_hi})")

cap.release()

cv2.destroyAllWindows()
