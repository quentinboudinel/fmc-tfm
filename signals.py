import numpy as np

def wave_packet_envelope(t, a, t_e):
    """Compute envelope of a wave packet.
    Parameters
    ----------
    t : float
        Time.
    a : float
        Max amplitude.
    t_e : float
        Packet emission duration.

    Returns
    -------
    out : float
        Envelope value at time t."""
    return a * (1 - np.cos(2*np.pi*t/t_e))

def wave_packet(t, a=1, t_e=0.000001, f=10000000):
    """Compute wave packet.
    Parameters
    ----------
    t : float
        Time.
    a : float
        Maximum amplitude.
    t_e : float
        Packet emission duration.
    f : float
        Packet frequency.

    Returns
    -------
    out : float
        Wave packet at time t."""
    if t>=0 and t<t_e:
        return wave_packet_envelope(t, a, t_e) * np.cos(2*np.pi*f*t)

    else:
        return 0

def main():
    pass

if __name__=='__main__':
    main()
