import numpy as np
from numpy.linalg import norm
from multiprocessing.pool import Pool

def ppi_to_ppm(pixel_density):
    """Returns pixels per meter pixel density converted from pixels per inch
    pixel density.
    Parameters
    ----------
    pixel_density : float
        Pixel density in pixels per inch.

    Returns
    -------
    out : float
        Pixel density in pixels per meter."""
    return pixel_density/0.0254

def parse_data(data):
    """
    Parameters
    ----------
    data : dict
        Dictionnary of Full Matrix Capture data.

    Returns
    -------
    out : float, float, float, list
        Celerity of sound in the material scanned, distance (step) between
        probes, sampling rate of the scan, full matrix capture data."""
    full_matrix_capture = data["full_matrix_capture"]
    sampling_rate = data["sampling_rate"]
    probes_step = data["probes_step"]
    celerity = data["celerity"]
    return celerity, probes_step, sampling_rate, full_matrix_capture

def get_sample(point, probe):
    """IN DEVELOPEMENT
    Parameters
    ----------
    point : numpy.array
    probe : numpy.array
        Dictionnary of Full Matrix Capture data.

    Returns
    -------
    out : ...
        ..."""
    try:
        return full_matrix_capture[t][r][int(
            (norm(point-transmitter) + norm(receiver-point)) *sampling_rate/celerity
        )]
    except IndexError:
        return 0.

class Tfm:
    """
    Class implementing functionalities for the Total Focusing Method.
    """
    DEFAULT = {
        "pixel_density": 1000
    }
    def __init__(self, pixel_density=DEFAULT["pixel_density"]):
        """
        Parameters
        ----------
        pixel_density : float

        Returns
        -------
        out : Tfm instance
            Total Focusing Method class instance defined with pixel density."""
        self.pixel_density = pixel_density

    def process(self, data):
        """Returns image computed with Total Focusing Method.
        Parameters
        ----------
        data : dict
            Data obtained with Full Matrix Capture

        Returns
        -------
        out : numpy.array
            Computed image."""
        celerity, probes_step, sampling_rate, \
        full_matrix_capture = parse_data(data)
        probes_number = len(full_matrix_capture)
        probes = [np.array([probes_step*(probe_number-probes_number/2), 0]) \
                  for probe_number in range(probes_number)]
        scan_depth = celerity*full_matrix_capture[0][0].size/(2*sampling_rate)
        x_size = int(scan_depth * self.pixel_density)
        y_size = int(scan_depth/2 * self.pixel_density)
        image = np.zeros((x_size, y_size))
        points = np.array([[np.array((x, y)) for y in np.linspace(0, scan_depth, y_size)] for x in np.linspace(-scan_depth, scan_depth, x_size)])

        indices = [(x, y) for x in range(x_size) for y in range(y_size)]

        with Pool() as pool:
            results = pool.starmap(add_sample, indices)
        return image

def main():
    # Import libraries required for tests
    import tkinter.filedialog
    import matplotlib.pyplot as plt
    import pickle

    # Ask to open file on which to test implemented total focusing method
    with tkinter.filedialog.askopenfile('rb') as data_file:
        # Loading the data
        data = pickle.load(data_file)

        # Instanciating the Tfm class object
        tfm = Tfm()

        # Compute image scale extent
        scan_depth = data["full_matrix_capture"][0][0].size*data["celerity"]/(2*data["sampling_rate"])
        extent = [-scan_depth/2, scan_depth/2, -scan_depth, scan_depth]

        # Compute and show image
        plt.imshow(tfm.process(data), extent=extent)
        plt.show()

if __name__=='__main__':
    main()
