<script>
	import { onMount } from 'svelte';

	onMount(() => {
		const links = Array.from(document.querySelectorAll('aside a'));

		links.forEach((link) => {
			link.addEventListener('click', (event) => {
				event.preventDefault();

				// @ts-ignore
				const targetId = event.target.getAttribute('href');
				const targetSection = document.querySelector(targetId);

				targetSection.scrollIntoView({ behavior: 'smooth' });
			});
		});
	});
</script>

<div class="container mx-auto px-4 py-8">
	<nav class="mb-8">
		<a href="/" class="text-green-400 hover:text-blue-300">Home</a>
	</nav>
	<header class="mb-12">
		<h1 class="text-6xl font-bold text-6xl font-bold text-green-600">DmailFi Whitepaper</h1>
	</header>
	<main class="grid grid-cols-3 gap-8">
		<div class="col-span-2">
			<section id="abstract" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">Abstract</h2>
				<p class="text-lg mb- 6 text-justify text-justify">
					In today's digital age, where communication and finance are pivotal to both personal and
					professional spheres, DmailFi emerges as a groundbreaking solution, merging email services
					with decentralized finance (DeFi) functionalities. Utilizing the robustness of blockchain
					technology, specifically the Internet Computer Protocol (ICP), DmailFi offers a secure,
					private, and efficient platform for conducting email communication intertwined with
					financial transactions. This white paper delves into existing challenges within email
					services and DeFi platforms, unveiling the innovative technology behind DmailFi, its
					unique features, practical applications, and a forward-looking roadmap.
				</p>
			</section>
			<section id="problem" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">The Problem</h2>
				<p class="mb- 6 text-justify">
					DmailFi addresses several critical issues plaguing traditional email services and DeFi
					platforms:
				</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>
						<strong class="text-green-500">Security Vulnerabilities:</strong> Traditional email services are susceptible to security
						breaches, such as Business Email Compromise (BEC), and lack native end-to-end encryption
						for emails at rest.
					</li>
					<li>
						<strong class="text-green-500">High Subscription Costs:</strong> Services like Google Workspace and Outlook impose significant
						subscription fees, particularly burdensome for organizations with a large number of
						employees.
					</li>
					<li>
						<strong class="text-green-500">Email Address Generation:</strong> There's a pressing need for generating a multitude of email
						addresses assignable to customers for auditing purposes, without requiring customers to
						forfeit their personal email addresses.
					</li>
					<li>
						<strong class="text-green-500">DeFi Integration:</strong> DeFi platforms often present user-unfriendly interfaces and exist
						separately from everyday communication tools, leading to inefficiencies and heightened
						security risks.
					</li>
				</ul>
			</section>
			<section id="solution" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">The Solution</h2>
				<p class="mb- 6 text-justify">DmailFi innovatively addresses these challenges</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>
						<strong class="text-green-500">Cryptographic Verification:</strong> Utilizes cryptographic signatures to authenticate the origin
						of emails, enhancing security.
					</li>
					<li>
						<strong class="text-green-500">ICP Platform:</strong> Leverages the low-cost infrastructure of the ICP platform for generating a
						vast number of email addresses.
					</li>
					<li>
						<strong class="text-green-500">Address Generation:</strong> There's a pressing need for generating a multitude of email
						addresses assignable to customers for auditing purposes, without requiring customers to
						forfeit their personal email addresses.
					</li>
					<li>
						<strong class="text-green-500">ICP vetKD:</strong> Employs ICP vetKD to encrypt emails in transit and at rest, ensuring superior
						security and privacy.
					</li>
				</ul>
			</section>
			<section id="architecture" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">Architecture</h2>
				<p class="mb- 6 text-justify">DmailFi's architecture is built on several key components...</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>
						<strong class="text-green-500">Master Canister:</strong> Serves as a registry linking domain names (e.g., user@example.com) to
						Mail Canister IDs and facilitates the creation of canisters for users.
					</li>
					<li>
						<strong class="text-green-500">Mail Canister:</strong> Functions as the storage unit for emails and files associated with a
						domain name (e.g., example.com).
					</li>
					<li>
						<strong class="text-green-500">Domain Verification Service (DVS):</strong> A Web2 API service confirming domain ownership,
						instructing the Master Canister to generate a Mail Canister for the domain.
					</li>
					<li>
						<strong class="text-green-500">Web3 SMTP Gateway:</strong> A gateway facilitating the reception of emails via the Simple Mail
						Transfer Protocol from traditional email environments to the ICP network.
					</li>
					<li>
                        <strong class="text-green-500">Frontend:</strong> Provides a workspace for domain owners to read and manage their emails.
                    </li>
				</ul>
			</section>
			<section id="registration" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">Registration Workflow</h2>
				<p class="mb- 6 text-justify">DmailFi simplifies the registration process...</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>Users submit a domain name for verification via DVS.</li>
					<li>
						Post-verification, DVS prompts the Master Canister to create a corresponding Mail
						Canister.
					</li>
					<li>Upon Mail Canister creation, users can commence sending emails to other domains.</li>
				</ul>
			</section>
			<section id="mail" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">Mail Sending Workflow</h2>
				<p class="mb- 6 text-justify">Email transmission in DmailFi operates seamlessly:</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>
						Sending users' private Mail Canisters consult the Master Canister to retrieve the
						recipient domain's Canister ID and direct emails accordingly.
					</li>
					<li>
						For recipient domains not on the ICP platform, the Mail Canister employs an HTTP
						Outgoing call to relay the email through the Web3 SMTP Gateway to external recipients.
					</li>
				</ul>
			</section>
			<section id="usecase" class="mb-12">
				<h2 class="text-3xl font-bold mb-4 text-green-600">Use Cases</h2>
				<p class="mb- 6 text-justify">
					DmailFi revolutionizes email communication and financial transactions for:
				</p>
				<ul class="list-disc ml-5 mb- 6 text-justify">
					<li>
						Freelancers: Streamlining invoice issuance and payment reception directly via email.
					</li>
					<li>Businesses: Enhancing management of payments and communications.</li>
					<li>
						Regulated Industries: Health, Government and Legal Industries can send encrypted mails.
					</li>
					<li>
						Individuals: Facilitating money transfers as effortlessly as sending an email, ideal for
						bill splitting or gifting.
					</li>
				</ul>
			</section>
		</div>
		<aside class="col-span-1">
			<div class="sticky top-8">
				<div class="mb-8">
					<h3 class="text-xl font-semibold mb-4 text-green-500">Table of Contents</h3>
					<ul class="space-y-2">
						<li><a href="#abstract" class="text-green-600 hover:text-green-300">Abstract</a></li>
						<li><a href="#intro" class="text-green-600 hover:text-green-300">Introduction</a></li>
						<li><a href="#problem" class="text-green-600 hover:text-green-300">The Problem</a></li>
						<li><a href="#solution" class="text-green-600 hover:text-green-300">The Solution</a></li>
						<li>
							<a href="#architecture" class="text-green-600 hover:text-green-300">Architecture</a>
						</li>
						<li>
							<a href="#registration" class="text-green-600 hover:text-green-300"
								>Registration Workflow</a
							>
						</li>
						<li>
							<a href="#mail" class="text-green-600 hover:text-green-300">Mail Sending Workflow</a>
						</li>
						<li><a href="#usecase" class="text-green-600 hover:text-green-300">Use Cases</a></li>
					</ul>
				</div>
			</div>
		</aside>
	</main>
</div>

<style>
	.active {
		color: #000;
	}
</style>
