import numpy as np
import signals as sg

class Fmc:
    """Full Matrix Capture class"""
    
    DEFAULT = {
        "probes_number": 5,
        "probes_step": 0.001,
        "scan_depth": 0.1,
        "sampling_rate": 40000000
    }

    def __init__(self, probes_number=DEFAULT["probes_number"],
                 probes_step=DEFAULT["probes_step"],
                 scan_depth=DEFAULT["scan_depth"],
                 sampling_rate=DEFAULT["sampling_rate"]):
        """Function to initialize a probe with parameters
        Params: - probes_number         number of probes                (1)
                - probes_step           step between probes             (m)
                - scan_depth            depth of the scan               (m)
                - sampling_rate         sampling rate of the signal     (Hz)
        """
        self.probes_number = probes_number
        self.probes_step = probes_step
        self.scan_depth = scan_depth
        self.sampling_rate = sampling_rate

        self.probes = [np.array([self.probes_step*(probe_number-(self.probes_number-1)/2), 0]) for probe_number in range(self.probes_number)]

    def capture(self, material):
        """Capture all the data of a Full Matrix Capture
        Parameters
        ----------
        material : Material class object
            Material class object to scan.

        Returns
        -------
        out : dict
            Full Matrix Capture data."""
        samples_number = int(2*self.sampling_rate*self.scan_depth/material.celerity)
        T = np.linspace(0, 2*self.scan_depth/material.celerity, samples_number)
        full_matrix_capture = [[np.zeros(samples_number) \
                                for _ in range(self.probes_number)] \
                                for _ in range(self.probes_number)]
        for t, transmitter in enumerate(self.probes):
            for r, receiver in enumerate(self.probes):
                for delay in material.delay(transmitter, receiver):
                    full_matrix_capture[t][r] += np.array([sg.wave_packet(t-delay) for t in T])

        return {"celerity": material.celerity,
                "probes_step": self.probes_step,
                "sampling_rate": self.sampling_rate,
                "full_matrix_capture": full_matrix_capture}

def main():
    import tkinter.filedialog
    import matplotlib.pyplot as plt
    import pickle

    from material import Material

    with tkinter.filedialog.asksaveasfile("wb") as data_file:
        defects = [np.array([0.02, 0.05])]
        material = Material(defects=defects)
        probes_number = 2
        fmc = Fmc(probes_number=probes_number)
        pickle.dump(fmc.capture(material), data_file)

    with tkinter.filedialog.askopenfile("rb") as data_file:
        data = pickle.load(data_file)
        sampling_rate = data["sampling_rate"]
        fmc = data["full_matrix_capture"]
        T = np.linspace(0, fmc[0][0].size/sampling_rate, fmc[0][0].size)
        plt.plot(T, fmc[0][0])
        plt.show()

if __name__=='__main__':
    main()
