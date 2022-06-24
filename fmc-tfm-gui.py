import tkinter as tk
import tkinter.filedialog
import pathlib
import pickle
import numpy as np
import matplotlib.pyplot as plt
from material import Material
from full_matrix_capture import Fmc
from total_focusing_method import ppi_to_ppm, Tfm

class MaterialCelerityFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.celerity_double_var = tk.DoubleVar(self, value=Material.DEFAULT["celerity"]/1000)
        self.celerity_label = tk.Label(self, text="Celerity of sound in the material (km/s):")
        self.celerity_label.pack(side=tk.LEFT)
        self.celerity_entry = tk.Entry(self, textvariable=self.celerity_double_var)
        self.celerity_entry.pack(side=tk.LEFT)

    def get(self):
        return self.celerity_double_var.get()*1000

class MaterialAddDefectFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.add_defect_label = tk.Label(self, text="Defect coordinates (cm, cm):")
        self.add_defect_label.pack(side=tk.LEFT)
        self.add_defect_x_double_var = tk.DoubleVar(self)
        self.add_defect_x_entry = tk.Entry(self, textvariable=self.add_defect_x_double_var, width=5)
        self.add_defect_x_entry.pack(side=tk.LEFT)
        self.add_defect_y_double_var = tk.DoubleVar(self)
        self.add_defect_y_entry = tk.Entry(self, textvariable=self.add_defect_y_double_var, width=5)
        self.add_defect_y_entry.pack(side=tk.LEFT)
        self.add_defect_button = tk.Button(self, text="Add defect", command=parent.add_defect)
        self.add_defect_button.pack(side=tk.LEFT)

    def get(self):
        return np.array([self.add_defect_x_double_var.get()/100, self.add_defect_y_double_var.get()/100])

class MaterialDefectsFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.defects = Material.DEFAULT["defects"]
        self.defects_var = tk.StringVar(value=self.defects)
        self.defects_label = tk.Label(self, text="List of defects:")
        self.defects_label.pack()
        self.defects_listbox = tk.Listbox(self, listvariable=self.defects_var, selectmode="extended")
        self.defects_listbox.pack()
        self.delete_selected_defects_button = tk.Button(self, text="Delete selected defects", command=self.delete_selected_defects)
        self.delete_selected_defects_button.pack()
        self.add_defect_frame = MaterialAddDefectFrame(self)
        self.add_defect_frame.pack()

    def add_defect(self):
        self.defects.append(self.add_defect_frame.get())
        self.defects_var.set(self.defects)

    def delete_selected_defects(self):
        self.defects = [defect for d, defect in enumerate(self.defects) if d not in self.defects_listbox.curselection()]
        self.defects_var.set(self.defects)

    def get(self):
        return self.defects

class MaterialLabelFrame(tk.LabelFrame):
    def __init__(self, parent):
        super().__init__(parent, text="Material characteristics")
        self.celerity_frame = MaterialCelerityFrame(self)
        self.celerity_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                 padx=10, pady=5)
        self.defects_frame = MaterialDefectsFrame(self)
        self.defects_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                       padx=10, pady=5)

    def get(self):
        return self.celerity_frame.get(), self.defects_frame.get()

class ProbesNumberFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.probes_number_int_var = tk.IntVar(self, value=Fmc.DEFAULT["probes_number"])
        self.probes_number_label = tk.Label(self, text="Number of probes:")
        self.probes_number_label.pack(side=tk.LEFT)
        self.probes_number_entry = tk.Entry(self, textvariable=self.probes_number_int_var)
        self.probes_number_entry.pack(side=tk.LEFT)

    def get(self):
        return self.probes_number_int_var.get()

class ProbesStepFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.probes_step_double_var = tk.DoubleVar(self, value=Fmc.DEFAULT["probes_step"]*1000)
        self.probes_step_label = tk.Label(self, text="Step between probes (mm):")
        self.probes_step_label.pack(side=tk.LEFT)
        self.probes_step_entry = tk.Entry(self, textvariable=self.probes_step_double_var)
        self.probes_step_entry.pack(side=tk.LEFT)

    def get(self):
        return self.probes_step_double_var.get()/1000

class ScanDepthFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.scan_depth_double_var = tk.DoubleVar(self, value=Fmc.DEFAULT["scan_depth"]*100)
        self.scan_depth_label = tk.Label(self, text="Scan depth (cm):")
        self.scan_depth_label.pack(side=tk.LEFT)
        self.scan_depth_entry = tk.Entry(self, textvariable=self.scan_depth_double_var)
        self.scan_depth_entry.pack(side=tk.LEFT)

    def get(self):
        return self.scan_depth_double_var.get()/100

class SamplingRateFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.sampling_rate_double_var = tk.DoubleVar(self, value=Fmc.DEFAULT["sampling_rate"]/10**6)
        self.sampling_rate_label = tk.Label(self, text="Sampling rate (MHz):")
        self.sampling_rate_label.pack(side=tk.LEFT)
        self.sampling_rate_entry = tk.Entry(self, textvariable=self.sampling_rate_double_var)
        self.sampling_rate_entry.pack(side=tk.LEFT)

    def get(self):
        return self.sampling_rate_double_var.get()*10**6

class ProbeLabelFrame(tk.LabelFrame):
    def __init__(self, parent):
        super().__init__(parent, text="Probe parameters")
        self.probes_number_frame = ProbesNumberFrame(self)
        self.probes_number_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                      padx=10, pady=5)
        self.probes_step_frame = ProbesStepFrame(self)
        self.probes_step_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                    padx=10, pady=5)
        self.scan_depth_frame = ScanDepthFrame(self)
        self.scan_depth_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                   padx=10, pady=5)
        self.sampling_rate_frame = SamplingRateFrame(self)
        self.sampling_rate_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                      padx=10, pady=5)

    def get(self):
        return self.probes_number_frame.get(), self.probes_step_frame.get(), \
               self.scan_depth_frame.get(), self.sampling_rate_frame.get()

class FmcLabelFrame(tk.LabelFrame):
    def __init__(self, parent):
        super().__init__(parent, text="Full Matrix Capture")
        self.material_label_frame = MaterialLabelFrame(self)
        self.material_label_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                       padx=10, pady=5)
        self.probe_label_frame = ProbeLabelFrame(self)
        self.probe_label_frame.pack(side=tk.TOP, expand=tk.YES, fill=tk.BOTH,
                                     padx=10, pady=5)
        self.capture_button = tk.Button(self, text="Capture",
                                        command=self.capture)
        self.capture_button.pack(side=tk.BOTTOM, padx=10, pady=5)

    def capture(self):
        celerity, defects = self.material_label_frame.get()
        material = Material(celerity, defects)
        probes_number, probes_step, scan_depth, \
        sampling_rate = self.probe_label_frame.get()
        probe = Fmc(probes_number, probes_step, scan_depth, sampling_rate)
        with tk.filedialog.asksaveasfile(mode="wb",
                                         initialdir=pathlib.Path(__file__)
                                                    .parent.absolute(),
                                         defaultextension=".fmc") as data_file:
            pickle.dump(probe.capture(material), data_file)

class PixelDensityFrame(tk.Frame):
    def __init__(self, parent):
        super().__init__(parent)
        self.pixel_density_double_var = tk.DoubleVar(self, value=10)
        self.pixel_density_label = tk.Label(self, text="Pixel density (ppi):")
        self.pixel_density_label.pack(side=tk.LEFT)
        self.pixel_density_entry = tk.Entry(self,
                                         textvariable=self
                                                      .pixel_density_double_var)
        self.pixel_density_entry.pack(side=tk.LEFT)

    def get(self):
        return self.pixel_density_double_var.get()

class TfmLabelFrame(tk.LabelFrame):
    def __init__(self, parent):
        super().__init__(parent, text="Total Focusing Method")
        self.pixel_density_frame = PixelDensityFrame(self)
        self.pixel_density_frame.pack(side=tk.TOP, fill=tk.BOTH)
        self.browse_and_process_button = tk.Button(self,
                                                   text="Browse data & Process",
                                                   command=self.process)
        self.browse_and_process_button.pack(side=tk.BOTTOM, padx=10, pady=5)

    def test_it(self):
        print(self.test_entry_string_var.get())

    def process(self):
        with tk.filedialog.askopenfile(mode="rb") as data_file:
            plt.imshow(Tfm(ppi_to_ppm(self.pixel_density_frame.get()))
                       .process(pickle.load(data_file)))
            plt.show()

class FmcTfmApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.fmc_label_frame = FmcLabelFrame(self)
        self.fmc_label_frame.pack(side=tk.LEFT, expand=tk.YES, fill=tk.BOTH)

        self.tfm_label_frame = TfmLabelFrame(self)
        self.tfm_label_frame.pack(side=tk.LEFT, expand=tk.YES, fill=tk.BOTH)

def main():
    FmcTfmApp().mainloop()

if __name__=="__main__":
    main()
