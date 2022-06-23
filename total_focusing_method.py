import numpy as np
from numpy.linalg import norm
from multiprocessing.pool import Pool

def ppp_to_ppm(resolution):
    return resolution/0.0254

def parse_data(data):
    full_matrix_capture = data["full_matrix_capture"]
    sampling_rate = data["sampling_rate"]
    probes_step = data["probes_step"]
    celerity = data["celerity"]
    return celerity, probes_step, sampling_rate, full_matrix_capture

def get_sample(point, probe):
    try:
        return full_matrix_capture[t][r][int(
            (norm(point-transmitter) + norm(receiver-point)) *sampling_rate/celerity
        )]
    except IndexError:
        return 0.

class Tfm:
    DEFAULT = {
        "resolution": 1000
    }
    def __init__(self, resolution=DEFAULT["resolution"]):
        self.resolution = resolution

    def process(self, data):
        celerity, probes_step, sampling_rate, \
        full_matrix_capture = parse_data(data)
        probes_number = len(full_matrix_capture)
        probes = [np.array([probes_step*(probe_number-probes_number/2), 0]) \
                  for probe_number in range(probes_number)]
        scan_depth = celerity*full_matrix_capture[0][0].size/(2*sampling_rate)
        x_size = int(scan_depth * self.resolution)
        y_size = int(scan_depth/2 * self.resolution)
        image = np.zeros((x_size, y_size))
        points = np.array([[np.array((x, y)) for y in np.linspace(0, scan_depth, y_size)] for x in np.linspace(-scan_depth, scan_depth, x_size)])

        indices = [(x, y) for x in range(x_size) for y in range(y_size)]

        with Pool() as pool:
            results = pool.starmap(add_sample, indices)
        return image

def main():
    import tkinter.filedialog
    import matplotlib.pyplot as plt
    import pickle

    with tkinter.filedialog.askopenfile('rb') as data_file:
        data = pickle.load(data_file)
        tfm = Tfm()
        resolution = tfm.resolution
        scan_depth = data["full_matrix_capture"][0][0].size*data["celerity"]/(2*data["sampling_rate"])
        extent = [-scan_depth/2, scan_depth/2, -scan_depth, scan_depth]
        plt.imshow(tfm.process(data), extent=extent)
        plt.show()

if __name__=='__main__':
    main()
