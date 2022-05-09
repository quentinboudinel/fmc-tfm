import numpy as np

def wave_packet_amplitude(t, a, t_e):
    """Function for the amplitude of a wave packet emitted at time = 0.
    Params: - t     time
            - a     maximum of the amplitude
            - t_e   time during which the packet is emitted
    """
    return a * (1 - np.cos(2*np.pi*t/t_e))

def wave_packet(t, a=1, t_e=1, f=10):
    """Function for a wave packet emitted at time = 0.
    Params: - t is the time
            - a     maximum of the amplitude (default=1)
            - t_e   time during which the packet is emitted (default=1s)
            - f     frequency of the wave contained in the packet (default=10Hz)
    """
    if t>=0 and t<t_e:
        return wave_packet_amplitude(t, a, t_e) * np.cos(2*np.pi*f*t)

    else:
        return 0

def main():
    import matplotlib.pyplot as plt
    T = np.linspace(-1, 2, 1000)
    WP = np.array([wave_packet(t) for t in T])
    plt.plot(T, WP)
    plt.show()

if __name__=='__main__':
    main()
