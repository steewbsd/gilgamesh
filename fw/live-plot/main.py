import serial
import struct
import numpy as np
import matplotlib.pyplot as plt
import matplotlib

matplotlib.use('gtk3agg')

def main():
    print("Opening serial port...")
    ser = serial.Serial('/dev/ttyUSB0', 115200, timeout=1)

    figure, (w_plot, x_plot, y_plot, z_plot) = plt.subplots(4, 1, figsize=(10, 6))
    time_axis = np.arange(0, 10, 0.1)
    w_data = np.zeros(time_axis.size)
    x_data = np.zeros(time_axis.size)
    y_data = np.zeros(time_axis.size)
    z_data = np.zeros(time_axis.size)
    
    # Starting plot
    w_line, = w_plot.plot(time_axis, w_data, label='W Data')
    x_line, = x_plot.plot(time_axis, x_data, label='X Data')
    y_line, = y_plot.plot(time_axis, y_data, label='Y Data')
    z_line, = z_plot.plot(time_axis, z_data, label='Z Data')

    w_plot.set(ylim=(-1, 1), yticks=np.arange(-1, 1, 50))
    x_plot.set(ylim=(-1, 1), yticks=np.arange(-1, 1, 50))
    y_plot.set(ylim=(-1, 1), yticks=np.arange(-1, 1, 50))
    z_plot.set(ylim=(-1, 1), yticks=np.arange(-1, 1, 50))

    w_plot.set_title('W Data')
    x_plot.set_title('X Data')
    y_plot.set_title('Y Data')
    z_plot.set_title('Z Data')

    plt.tight_layout()
    plt.ion()  # Interactive mode on
    plt.show()

    i = 0 
    loop = True
    while loop:
        try:
            read_bytes = ser.read(16)
            
            if len(read_bytes) == 16:
                w = struct.unpack('f', read_bytes[0:4])[0]
                x = struct.unpack('f', read_bytes[4:8])[0]
                y = struct.unpack('f', read_bytes[8:12])[0]
                z = struct.unpack('f', read_bytes[12:16])[0]

                # shift the data and append the new values
                w_data = np.roll(w_data, -1)
                x_data = np.roll(x_data, -1)
                y_data = np.roll(y_data, -1)
                z_data = np.roll(z_data, -1)

                w_data[-1] = w
                y_data[-1] = x
                x_data[-1] = y
                z_data[-1] = z

                w_line.set_ydata(w_data)
                x_line.set_ydata(x_data)
                y_line.set_ydata(y_data)
                z_line.set_ydata(z_data)

                # Redraw the plots
                plt.draw()
                plt.pause(0.01)  # Pause to allow the plot to update
        except KeyboardInterrupt:
            print("Exiting...")
            loop = False

if __name__ == "__main__":
    main()
