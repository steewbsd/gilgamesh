pkg load matgeom
pkg load instrument-control

graphics_toolkit("qt")

tmtry_serial = serial("/dev/ttyUSB0", 9600);
srl_flush(tmtry_serial);

center = [0 0 0];
% specify lengths (x y z)
sizes = [4 2 1];
% specify yaw pitch and roll
ypr = [0 0 0];

figure;
view (3);
axis equal;
hold off;

x = [1 0 0];
y = [0 1 0];
z = [0 0 1];

while true
  data = srl_read(tmtry_serial, 12);

  ## convert [u8; 4] array to float
  ## y = swapbytes (typecast(uint8(data(1:4)), "single"))
  ## p = swapbytes (typecast(uint8(data(5:8)), "single"))
  ## r = swapbytes (typecast(uint8(data(9:12)), "single"))
  y = typecast(uint8(data(1:4)), "single");
  p = typecast(uint8(data(5:8)), "single");
  r = typecast(uint8(data(9:12)), "single")

  ## rotate(c, x, r);
  ## rotate(c, y, p);
  ## rotate(c, z ,y);
  ypr = [y p r];
  clf
  drawCuboid ([center sizes ypr]);
  view (3);
  ## axis([0 5 0 5]);
  axis equal;
  drawnow;
  
  ## pause(0.5);
end

close
