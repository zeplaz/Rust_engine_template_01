from PySide6.QtWidgets import QApplication, QStyle
from PySide6.QtGui import QPixmap, QImageWriter
from PySide6.QtCore import QByteArray, QBuffer
import base64

import os 

app = QApplication([])

# Retrieve the icon and convert it to QPixmap
icon = app.style().standardIcon(QStyle.SP_VistaShield)
pixmap = icon.pixmap(16, 16)  # You can adjust the size as needed

# Save QPixmap to QByteArray in PNG format
byte_array = QByteArray()
buffer = QBuffer(byte_array)
buffer.open(QBuffer.WriteOnly)
pixmap.save(buffer, "PNG")

# Convert QByteArray to Base64 and print
base64_data = base64.b64encode(byte_array.data()).decode("utf-8")
print(base64_data)

dirname = os.path.dirname(os.path.realpath(__file__))

file_name = os.path.join(dirname, "icon_dump.txt")
open(file_name, "w").write(base64_data)