import { strictEqual } from "assert"
import {
    newClient,
    writeArtifact,
    readArtifact,
    instantiateContract,
    queryContract,
    uploadContract
} from './helpers.js'

const CONTRACT_LABEL = "Astroport ASTRO"
const CW20_BINARY_PATH = process.env.CW20_BINARY_PATH! || 'astroport_artifacts/astroport_token.wasm'
const TOKEN_INITIAL_AMOUNT = process.env.TOKEN_INITIAL_AMOUNT! || String(1_100_000_000_000000)

// Main
async function main() {
    const {terra, wallet} = newClient()
    const network = readArtifact(terra.config.chainID)
    console.log(`chainID: ${terra.config.chainID} wallet: ${wallet.key.accAddress}`)

    // Upload contract code
    network.tokenCodeID = await uploadContract(terra, wallet, CW20_BINARY_PATH!)
    console.log(`Token codeId: ${network.tokenCodeID}`)
    // Token info
    const TOKEN_NAME = "Astroport"
    const TOKEN_SYMBOL = "ASTRO"
    const TOKEN_DECIMALS = 6

    const TOKEN_INFO = {
        name: TOKEN_NAME,
        symbol: TOKEN_SYMBOL,
        decimals: TOKEN_DECIMALS,
        initial_balances: [
            {
                address: wallet.key.accAddress,
                amount: TOKEN_INITIAL_AMOUNT
            }
        ],
        marketing: {
            project: "Astroport",
            description: "Astroport is a neutral marketplace where anyone, from anywhere in the galaxy, can dock to trade their wares.",
            marketing: wallet.key.accAddress,
            logo: {
                url: "https://astroport.fi/astro_logo.svg"
            }
        }
    }

    // Instantiate astro token contract
    let resp = await instantiateContract(terra, wallet, wallet.key.accAddress, network.tokenCodeID, TOKEN_INFO, CONTRACT_LABEL)

    // @ts-ignore
    network.tokenAddress = resp.shift().shift()
    console.log("astro:", network.tokenAddress)
    console.log(await queryContract(terra, network.tokenAddress, { token_info: {} }))
    console.log(await queryContract(terra, network.tokenAddress, { minter: {} }))

    let balance = await queryContract(terra, network.tokenAddress, { balance: { address: TOKEN_INFO.initial_balances[0].address } })
    strictEqual(balance.balance, TOKEN_INFO.initial_balances[0].amount)

    writeArtifact(network, terra.config.chainID)
    console.log('FINISH')
}
main().catch(console.log)
