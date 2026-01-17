import serial
import struct
import numpy as np
import matplotlib.pyplot as plt
import matplotlib

matplotlib.use('gtk3agg')

def main():
    print("Opening serial port...")
    ser = serial.Serial('/dev/ttyUSB0', 115200, timeout=1)

    figure, (y_plot, p_plot, r_plot) = plt.subplots(3, 1, figsize=(10, 6))
    time_axis = np.arange(0, 10, 0.1)
    y_data = np.zeros(time_axis.size)
    p_data = np.zeros(time_axis.size)
    r_data = np.zeros(time_axis.size)
    
    # Starting plot
    y_line, = y_plot.plot(time_axis, y_data, label='Y Data')
    p_line, = p_plot.plot(time_axis, p_data, label='P Data')
    r_line, = r_plot.plot(time_axis, r_data, label='R Data')

    y_plot.set(ylim=(-190, 190), yticks=np.arange(-190, 190, 50))
    p_plot.set(ylim=(-190, 190), yticks=np.arange(-190, 190, 50))
    r_plot.set(ylim=(-190, 190), yticks=np.arange(-190, 190, 50))

    y_plot.set_title('Y Data')
    p_plot.set_title('P Data')
    r_plot.set_title('R Data')

    plt.tight_layout()
    plt.ion()  # Interactive mode on
    plt.show()

    i = 0 
    loop = True
    while loop:
        try:
            read_bytes = ser.read(12)
            
            if len(read_bytes) == 12:
                p = struct.unpack('f', read_bytes[0:4])[0]
                y = struct.unpack('f', read_bytes[4:8])[0]
                r = struct.unpack('f', read_bytes[8:12])[0]

                # shift the data and append the new values
                y_data = np.roll(y_data, -1)
                p_data = np.roll(p_data, -1)
                r_data = np.roll(r_data, -1)

                y_data[-1] = y
                p_data[-1] = p
                r_data[-1] = r

                y_line.set_ydata(y_data)
                p_line.set_ydata(p_data)
                r_line.set_ydata(r_data)

                # Redraw the plots
                plt.draw()
                plt.pause(0.01)  # Pause to allow the plot to update
        except KeyboardInterrupt:
            print("Exiting...")
            loop = False

if __name__ == "__main__":
    main()
