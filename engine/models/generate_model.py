
import torch
import torch.nn as nn
import torch.nn.functional as F

class LITSNet (nn.Module):
#
    def __init__ (self):
    #
        super(LITSNet, self).__init__()

        self.m_convol_0 = nn.Conv2d(   5,   15,    2)
        self.m_convol_1 = nn.Conv2d(  15,   25,    2)

        self.m_policy_0 = nn.Linear(1600, 1600)
        self.m_policy_1 = nn.Linear(1600, 1293)

        self.m_values_0 = nn.Linear(1600,  256)
        self.m_values_1 = nn.Linear( 256,    1)
    #

    def forward (self, x):
    #
        # Convolutional path.

        x = F.relu(self.m_convol_0(x))
        x = F.relu(self.m_convol_1(x))
        x = torch.flatten(x, 1)

        # Policy head.

        p = F.relu(self.m_policy_0(x))
        p = F.relu(self.m_policy_1(p))

        # Values head.

        v = F.relu(self.m_values_0(x))
        v = F.relu(self.m_values_1(v))

        return (p, v)
    #
#

# Create an instance of this model and save it to the template.pt file.

model = LITSNet()
model.eval()

trace = torch.jit.trace(model, torch.rand(1, 5, 10, 10))
trace.save("template.pt")
