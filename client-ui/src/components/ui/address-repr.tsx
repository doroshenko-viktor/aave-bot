import { Tooltip } from "./tooltip";

export type Props = {
  children: string;
}

export default function AddressRepr({ children: address }: Props) {

  return (
    <Tooltip
      content={`0x${address}`}
      positioning={{ placement: "top" }}
    >
      <span>
        {address.length > 6 ?
          `0x${address.substring(0, 5)}...${address.substring(address.length - 6, address.length - 1)}` :
          `0x${address}`
        }
      </span>
    </Tooltip>
  );
}
