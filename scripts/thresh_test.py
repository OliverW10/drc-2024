import cv2 as cv
import numpy as np
 
img = cv.imread('/home/oliver/car/drc-2024/client/images/image-2024-07-10 1:10:42.474617826 +00:00:00.png')
# img = cv.medianBlur(img,5)
img[:, :, 1] = 0
img[:, :, 2] = 0
blue = img[:, :, 0]
 
th2 = cv.adaptiveThreshold(blue,255,cv.ADAPTIVE_THRESH_MEAN_C, cv.THRESH_BINARY,25,-1)
th3 = cv.adaptiveThreshold(blue,255,cv.ADAPTIVE_THRESH_GAUSSIAN_C,\
 cv.THRESH_BINARY,11,2)
 
titles = ['Original Image', 'Global Thresholding (v = 127)',
 'Adaptive Mean Thresholding', 'Adaptive Gaussian Thresholding']
images = [img, th2, th3]
 
cv.imshow("img", img)
cv.imshow("b", blue)
cv.imshow("1", th2)
cv.imshow("2", th3)
cv.waitKey(0)