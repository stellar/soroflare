# Soroflare
> Careful:  
> This repository is an early stage of development.   
> It is not recommended to use this code in an production enviroment!

This repository contains the environment and virtual machine running as the backbone for the [FCA00C][fca00c] contest.  

## Virtual Machine 
The virtual machine contained in [soroflare-vm] is designed as a standalone rust crate.  
This allows an easy implementation of Soroban contract execution in arbitrary applications.

## FCA00C backend
A modified version of the actual [FCA00C][fca00c] backend is given in [soroflare-wrangler].  
The backend is built using the Cloudflare Wrangler stack and uses the [worker-rs] framework to compile to
WebAssembly.  
The [soroflare-wrangler] can is as an exemplary implementation of the [soroflare-vm].


[fca00c]: https://fca00c.com
[soroflare-vm]: ./soroflare-vm/
[soroflare-wrangler]: ./soroflare-wrangler/
[worker-rs]: https://github.com/cloudflare/workers-rs