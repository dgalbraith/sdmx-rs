<details>
<summary>XSD contract: <code>NameableType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="NameableType" abstract="true">
		<xs:annotation>
			<xs:documentation>NameableType is an abstract base type for all nameable objects.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="IdentifiableType">
				<xs:sequence>
					<xs:element ref="Name" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>Name provides for a human-readable name for the object. This may be provided in multiple, parallel language-equivalent forms.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element ref="Description" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>Description provides for a longer human-readable description of the object. This may be provided in multiple, parallel language-equivalent forms.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
