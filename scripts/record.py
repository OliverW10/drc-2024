import cv2
import time
import os
 
if os.path.isfile("video.avi"):
    raise Exception("file already exists, download it and delete it. `scp pi@raspberrypi.local:~/drc-2024/scripts/video.avi ~`")

cap = cv2.VideoCapture(0)
cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)


fps = cap.get(cv2.CAP_PROP_FPS)
fourcc = cv2.VideoWriter_fourcc(*'XVID')
out = cv2.VideoWriter(f'video.avi', fourcc, fps, (640, 480))

counter = 0
since = time.time()

try:
    while cap.isOpened():
        ret, frame = cap.read()
        if not ret:
            print("Can't receive frame (stream end?). Exiting ...")
            break
        
        out.write(frame)

        counter += 1
        if counter % 10 == 0:
            print(f"Recoding, {round(10/(time.time() - since), 2)} fps")
            since = time.time()

except KeyboardInterrupt:
    print("cleaning up")
 
# Release everything if job is finished
cap.release()
out.release()
# cv2.destroyAllWindows()
