import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torchvision import datasets, transforms
from torch.optim.lr_scheduler import StepLR
import optuna

from hyperparameter import param_scope, auto_param, lazy_dispatch


@auto_param
class Backbone(nn.Module):
    def __init__(
        self,
        chn1=32,
        chn2=64,
        ker1_size=3,
        ker2_size=3,
        activation="relu",
    ) -> None:
        super().__init__()
        self.conv1 = nn.Conv2d(1, chn1, ker1_size, 1)
        self.conv2 = nn.Conv2d(chn1, chn2, ker2_size, 1)
        self.activation = getattr(F, activation)

    def forward(self, x):
        x = self.conv1(x)
        x = self.activation(x)
        x = self.conv2(x)
        x = self.activation(x)
        x = F.max_pool2d(x, 2)
        return torch.flatten(x, 1)


@auto_param
class Head(nn.Module):
    def __init__(
        self,
        fc1=128,
        fc2=128,
        activation="relu",
        dropout=0.25,
    ) -> None:
        super().__init__()
        self.fc1 = nn.LazyLinear(fc1)
        self.fc2 = nn.LazyLinear(fc2)
        self.dropout1 = nn.Dropout(dropout)
        self.dropout2 = nn.Dropout(dropout)
        self.activation = getattr(F, activation)

    def forward(self, x):
        x = self.dropout1(x)
        x = self.fc1(x)
        x = self.activation(x)
        x = self.dropout2(x)
        return self.fc2(x)


class Net(nn.Module):
    def __init__(self) -> None:
        super().__init__()
        self.backbone = Backbone()
        self.head = Head()

    def forward(self, x):
        x = self.backbone(x)
        x = self.head(x)
        return F.log_softmax(x, dim=1)


def train(model, train_loader, optimizer):
    model.train()
    for batch_idx, (data, target) in enumerate(train_loader):
        optimizer.zero_grad()
        output = model(data)
        loss = F.nll_loss(output, target)
        loss.backward()
        optimizer.step()
        if batch_idx % 100 == 0:
            print(
                "Train Epoch: [{}/{} ({:.0f}%)]\tLoss: {:.6f}".format(
                    batch_idx * len(data),
                    len(train_loader.dataset),
                    100.0 * batch_idx / len(train_loader),
                    loss.item(),
                )
            )


def test(model, test_loader):
    model.eval()
    test_loss = 0
    correct = 0
    with torch.no_grad():
        for data, target in test_loader:
            output = model(data)
            test_loss += F.nll_loss(
                output, target, reduction="sum"
            ).item()  # sum up batch loss
            pred = output.argmax(
                dim=1, keepdim=True
            )  # get the index of the max log-probability
            correct += pred.eq(target.view_as(pred)).sum().item()
    test_loss /= len(test_loader.dataset)
    print(
        "\nTest set: Average loss: {:.4f}, Accuracy: {}/{} ({:.0f}%)\n".format(
            test_loss,
            correct,
            len(test_loader.dataset),
            100.0 * correct / len(test_loader.dataset),
        )
    )
    return test_loss / len(test_loader.dataset)


@auto_param
def train_model(batch_size=128, epochs=1, lr=1.0, momentum=0.9, step_lr_gamma=0.7):
    torch.manual_seed(0)
    transform = transforms.Compose(
        [transforms.ToTensor(), transforms.Normalize((0.1307,), (0.3081,))]
    )
    dataset1 = datasets.MNIST("../data", train=True, download=True, transform=transform)
    dataset2 = datasets.MNIST("../data", train=False, transform=transform)
    train_loader = torch.utils.data.DataLoader(dataset1, batch_size=batch_size)
    test_loader = torch.utils.data.DataLoader(dataset2, batch_size=batch_size)

    model = Net()
    optimizer = optim.SGD(model.parameters(), lr=lr, momentum=momentum)

    scheduler = StepLR(optimizer, step_size=1, gamma=step_lr_gamma)
    for epoch in range(1, epochs + 1):
        train(model, train_loader, optimizer)
        scheduler.step()
    return test(model, test_loader)


def wrapper(trial):
    trial = lazy_dispatch(trial)
    with param_scope(
        **{
            "train_model.lr": trial.suggest_categorical("train_model.lr", [0.1, 0.01]),
            "train_model.momentum": trial.suggest_categorical(
                "train_model.momentum", [0.9, 0.85]
            ),
            "Backbone.ker1_size": trial.suggest_categorical(
                "Backbone.ker1_size", [3, 5]
            ),
            "Head.dropout": trial.suggest_categorical("Head.dropout", [0.25, 0.15]),
        }
    ):
        return train_model()


study = optuna.create_study()
study.optimize(wrapper, n_trials=10)

optuna.visualization.plot_contour(study)
optuna.visualization.plot_parallel_coordinate(study)
