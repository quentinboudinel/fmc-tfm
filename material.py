import numpy as np

class Material:
    DEFAULT = {
        "celerity": 5600, #sound celerity in steel
        "defects": []
    }

    def __init__(self, celerity=DEFAULT["celerity"],
                 defects=DEFAULT["defects"]):
        """Initialize material.
        Params: - celerity  celerity of sound in the material           (km/s)
                - defects   list of defect positions in the material    (list)
        """
        self.celerity = celerity
        self.defects = defects

    def delay(self, transmitter, receiver):
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
