import cv2
import numpy as np
import os
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import LabelEncoder
from keras.models import Sequential
from keras.layers import Dense, Flatten

# Function to extract features from the image
def extract_features(image):
    resized_image = cv2.resize(image, (50, 50))  # Resize image to a constant size
    return resized_image.flatten()

# Load images and corresponding labels
def load_data(dataset_dir):
    X = []
    y = []
    for root, dirs, files in os.walk(dataset_dir):
        for file in files:
            if file.endswith(".jpg") or file.endswith(".png"):
                image_path = os.path.join(root, file)
                label = os.path.basename(root)
                image = cv2.imread(image_path)
                black_mask = cv2.inRange(image, (0, 0, 0), (50, 50, 50))  # Thresholding for black color
                contours, _ = cv2.findContours(black_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
                for contour in contours:
                    x, y, w, h = cv2.boundingRect(contour)
                    arrow_roi = image[y:y+h, x:x+w]
                    X.append(extract_features(arrow_roi))
                    y.append(label)
    return np.array(X), np.array(y)

# Load dataset
dataset_dir = "path_to_your_dataset_directory"
X, y = load_data(dataset_dir)

# Encode labels
label_encoder = LabelEncoder()
y_encoded = label_encoder.fit_transform(y)

# Split data into train and test sets
X_train, X_test, y_train, y_test = train_test_split(X, y_encoded, test_size=0.2, random_state=42)

# Build neural network model
model = Sequential()
model.add(Dense(128, input_shape=(2500,), activation='relu'))
model.add(Dense(64, activation='relu'))
model.add(Dense(3, activation='softmax'))  # 3 classes: Left, Right, None

# Compile model
model.compile(loss='sparse_categorical_crossentropy', optimizer='adam', metrics=['accuracy'])

# Train model
model.fit(X_train, y_train, epochs=10, batch_size=32, validation_split=0.1)

# Evaluate model
loss, accuracy = model.evaluate(X_test, y_test)
print(f'Test Loss: {loss}, Test Accuracy: {accuracy}'
