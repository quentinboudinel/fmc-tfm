import numpy as np

class Material:
    """Class representing a material to be scanned"""
    
    DEFAULT = {
        "celerity": 5600, #sound celerity in steel
        "defects": []
    }

    def __init__(self, celerity=DEFAULT["celerity"],
                 defects=DEFAULT["defects"]):
        """
        Parameters
        ----------
        celerity : float
            Celerity of sound in the material.
        defects : list
            List of defects coordinates represented by numpy.array.

        Returns
        -------
        out : Material class object
            Material instance with defined celerity and defects."""
        self.celerity = celerity
        self.defects = defects

    def delay(self, transmitter, receiver):
        """Compute delays of signal from transmitter to receiver.
        Parameters
        ----------
        transmitter : numpy.array
            Coordiantes of the transmitter.
        receiver : numpy.array
            Coordinates of the receiver.

        Returns
        -------
        out : list
            List of delays of reflections on defects."""
        delays = []
        for deffect in self.defects:
            d1 = np.linalg.norm(deffect - transmitter)
            d2 = np.linalg.norm(receiver - deffect)
            delay = (d1 + d2)/self.celerity
            delays.append(delay)
            print("Transmitter:", transmitter)
            print("Deffect:", deffect)
            print("Receiver:", receiver)
            print("Delay:", delay)
            print()
        return delays

def main():
    pass

if __name__=='__main__':
    main()
