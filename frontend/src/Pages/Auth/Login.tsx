import Logo from '@/Shared/Logo'
import TextInput from '@/Shared/TextInput'
import { FormEvent, useCallback } from 'react'
import LoadingButton from '@/Shared/LoadingButton'
import { Head, useForm, usePage } from '@inertiajs/react'

const Login = () => {
	const page = usePage()
	console.log(page.props)
	const { data, setData, post, processing, errors } = useForm({
		email: 'johndoe@example.com',
		password: 'secret',
	})

	const login = useCallback(
		(event: FormEvent<HTMLFormElement>) => {
			event.preventDefault()

			post('/auth/login')
		},
		[post]
	)

	return (
		<>
			<Head title="Login" />
			<div className="flex items-center justify-center p-6 min-h-screen bg-indigo-800">
				<div className="w-full max-w-md">
					<Logo className="block mx-auto w-full max-w-xs fill-white" height="50" />
					<form onSubmit={login} className="mt-8 bg-white rounded-lg shadow-xl overflow-hidden">
						<div className="px-10 py-12">
							<h1 className="text-center text-3xl font-bold">Welcome Back!</h1>
							<div className="mt-6 mx-auto w-24 border-b-2" />
							<TextInput
								value={data.email}
								onChange={email => setData('email', email)}
								error={errors.email}
								className="mt-10"
								label="Email"
								type="email"
								autoFocus
								autoCapitalize="off"
							/>
							<TextInput
								value={data.password}
								onChange={password => setData('password', password)}
								error={errors.password}
								className="mt-6"
								label="Password"
								type="password"
							/>
						</div>
						<div className="flex px-10 py-4 bg-gray-100 border-t border-gray-100">
							<LoadingButton loading={processing} className="btn-indigo ml-auto" type="submit">
								Login
							</LoadingButton>
						</div>
					</form>
				</div>
			</div>
		</>
	)
}

export default Login
