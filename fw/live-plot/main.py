import serial
import struct

# Open connection
def main():
    print("Opening serial port...")
    ser = serial.Serial('/dev/ttyUSB0', 9600, timeout=1)
    while True:
        read_bytes = ser.read(12)
        y = struct.unpack('f', read_bytes[0:4])[0]
        p = struct.unpack('f', read_bytes[4:8])[0]
        r = struct.unpack('f', read_bytes[8:12])[0]
        print(f"{y},{p},{r}")
    

if __name__ == "__main__":
    main()
