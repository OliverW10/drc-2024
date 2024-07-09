import cv2
import os
import numpy as np
import glob

# Get the current working directory
path = os.getcwd()

# Define input and output directories
inputPar = os.path.join(path, '../client/images/image*.png')

outPar = os.path.join(path, 'Output/')
os.makedirs(outPar, exist_ok=True)

# List all files in the input directory
files = glob.glob("/home/oliver/car/drc-2024/client/images/image*.png") #os.listdir(inputPar)
# Loop through each file in the input directory
for file in files:
    fitem = os.path.join(inputPar, file)
    image = cv2.imread(fitem)

    # Grayscale conversion
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)

    # Canny edge detection
    edges = cv2.Canny(gray, 100, 140, apertureSize=3)
    cv2.imshow("edges", edges)
    cv2.waitKey()
    continue
    # Hough Line Transform
    lines = cv2.HoughLines(edges, 1.5, np.pi / 180, 200)

    # Iterate through each detected line
    for line in lines:
        rho, theta = line[0]
        # Convert polar coordinates to Cartesian coordinates
        a = np.cos(theta)
        b = np.sin(theta)
        x0 = a * rho
        y0 = b * rho
        x1 = int(x0 + 1000 * (-b))
        y1 = int(y0 + 1000 * (a))
        x2 = int(x0 - 1000 * (-b))
        y2 = int(y0 - 1000 * (a))

        # Draw lines on the original image
        cv2.line(image, (x1, y1), (x2, y2), (255, 0, 0), 2)

    # Define the output file path
    fout = os.path.join(outPar, file)

    # Save the grayscale image with detected edges
    cv2.imwrite(fout, gray)